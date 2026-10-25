//! Sonde GPUI (T047) : fenêtre à barre de titre dessinée, contrôles accessibles,
//! icône de notification, mouvement réduit et sélection d'un exécutable.
//!
//! `--smoke` ouvre la fenêtre, la détruit, la rouvre puis quitte.

#[cfg(windows)]
fn main() {
    probe::run(std::env::args().any(|argument| argument == "--smoke"));
}

#[cfg(not(windows))]
fn main() {
    eprintln!("La sonde GPUI ne fonctionne que sous Windows.");
}

#[cfg(windows)]
mod probe {
    use std::cell::Cell;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::time::Duration;

    use gpui::{
        AnimationExt as _, App, Bounds, Context, FocusHandle, KeyBinding, PathPromptOptions,
        QuitMode, Role, SharedString, SpringAnimation, SpringConfig, Toggled, Window, WindowBounds,
        WindowControlArea, WindowHandle, WindowId, WindowOptions, actions, div, prelude::*, px,
        rgb, size,
    };
    use power_plan_sync::windows::file_dialog::choose_executable;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    };
    use windows::core::BOOL;

    actions!(gpui_probe, [Tab, TabPrev]);

    const SOFTWARE: [&str; 3] = ["Battlefield 6", "Blender", "OBS Studio"];
    const SWITCH_SPRING: SpringConfig = SpringConfig::new(380.0, 30.0, 1.0);

    pub fn run(smoke: bool) {
        gpui_platform::application()
            .with_quit_mode(QuitMode::Explicit)
            .run(move |cx: &mut App| {
                cx.bind_keys([
                    KeyBinding::new("tab", Tab, None),
                    KeyBinding::new("shift-tab", TabPrev, None),
                ]);
                cx.set_reduce_motion(!client_area_animation_enabled());

                let (tray, open_id, quit_id) = match build_tray() {
                    Ok(tray) => tray,
                    Err(error) => {
                        eprintln!("Icône de notification impossible : {error}");
                        cx.quit();
                        return;
                    }
                };

                // Fenêtre ouverte, effacée par GPUI à sa fermeture. Un `update` refusé ne
                // prouve pas une fermeture : l'application peut être occupée.
                let open_window: Rc<Cell<Option<WindowId>>> = Rc::default();
                let closed = Rc::clone(&open_window);
                cx.on_window_closed(move |_, id| {
                    if closed.get() == Some(id) {
                        closed.set(None);
                    }
                })
                .detach();

                let first = open_probe_window(cx, &open_window);
                cx.spawn(async move |cx| {
                    // L'icône vit tant que la boucle tourne.
                    let _tray: TrayIcon = tray;
                    let mut window = first;
                    if smoke {
                        cx.background_executor()
                            .timer(Duration::from_millis(400))
                            .await;
                        close_probe_window(window.take(), cx);
                        cx.background_executor()
                            .timer(Duration::from_millis(400))
                            .await;
                        window = cx.update(|cx| open_probe_window(cx, &open_window));
                        cx.background_executor()
                            .timer(Duration::from_millis(400))
                            .await;
                        close_probe_window(window.take(), cx);
                        cx.update(|cx| cx.quit());
                        return;
                    }
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(100))
                            .await;
                        let wants_open = TrayIconEvent::receiver().try_iter().any(|event| {
                            matches!(
                                event,
                                TrayIconEvent::Click {
                                    button: MouseButton::Left,
                                    button_state: MouseButtonState::Up,
                                    ..
                                }
                            )
                        });
                        let mut wants_quit = false;
                        let mut open_from_menu = false;
                        for event in MenuEvent::receiver().try_iter() {
                            wants_quit |= event.id == quit_id;
                            open_from_menu |= event.id == open_id;
                        }
                        if wants_quit {
                            close_probe_window(window.take(), cx);
                            cx.update(|cx| cx.quit());
                            return;
                        }
                        if open_window.get().is_none() {
                            window = None;
                        }
                        if (wants_open || open_from_menu) && window.is_none() {
                            window = cx.update(|cx| open_probe_window(cx, &open_window));
                        }
                    }
                })
                .detach();
            });
    }

    fn build_tray() -> Result<(TrayIcon, tray_icon::menu::MenuId, tray_icon::menu::MenuId), String>
    {
        let open = MenuItem::new("Ouvrir la sonde", true, None);
        let quit = MenuItem::new("Quitter la sonde", true, None);
        let menu = Menu::new();
        menu.append(&open).map_err(|error| error.to_string())?;
        menu.append(&quit).map_err(|error| error.to_string())?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("Sonde GPUI PowerPlanSync")
            .with_icon(tray_icon_image()?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|error| error.to_string())?;
        Ok((tray, open.id().clone(), quit.id().clone()))
    }

    fn tray_icon_image() -> Result<Icon, String> {
        const SIDE: u32 = 16;
        let mut rgba = Vec::with_capacity((SIDE * SIDE * 4) as usize);
        for y in 0..SIDE {
            for x in 0..SIDE {
                let inside = (2..14).contains(&x) && (2..14).contains(&y);
                rgba.extend_from_slice(if inside {
                    &[0x2f, 0x6f, 0xeb, 0xff]
                } else {
                    &[0, 0, 0, 0]
                });
            }
        }
        Icon::from_rgba(rgba, SIDE, SIDE).map_err(|error| error.to_string())
    }

    fn open_probe_window(
        cx: &mut App,
        open_window: &Cell<Option<WindowId>>,
    ) -> Option<WindowHandle<ProbeView>> {
        let bounds = Bounds::centered(None, size(px(520.), px(440.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Sonde GPUI".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        match cx.open_window(options, |window, cx| {
            cx.new(|cx| ProbeView::new(window, cx))
        }) {
            Ok(handle) => {
                open_window.set(Some(handle.window_id()));
                cx.activate(true);
                Some(handle)
            }
            Err(error) => {
                eprintln!("Ouverture de la fenêtre impossible : {error}");
                None
            }
        }
    }

    fn close_probe_window(window: Option<WindowHandle<ProbeView>>, cx: &mut gpui::AsyncApp) {
        if let Some(window) = window {
            let _ = window.update(cx, |_, window, _| window.remove_window());
        }
    }

    /// « Effets d'animation » de Windows. Une lecture échouée garde le mouvement.
    fn client_area_animation_enabled() -> bool {
        let mut enabled = BOOL(1);
        let read = unsafe {
            SystemParametersInfoW(
                SPI_GETCLIENTAREAANIMATION,
                0,
                Some((&mut enabled as *mut BOOL).cast()),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
        };
        read.is_err() || enabled.as_bool()
    }

    fn window_hwnd(window: &Window) -> Option<HWND> {
        match HasWindowHandle::window_handle(window).ok()?.as_raw() {
            RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as *mut _)),
            _ => None,
        }
    }

    struct ProbeView {
        focus_handle: FocusHandle,
        following: bool,
        selected: usize,
        status: SharedString,
    }

    impl ProbeView {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                focus_handle,
                following: true,
                selected: 0,
                status: "Prêt".into(),
            }
        }

        fn set_status(&mut self, status: impl Into<SharedString>, cx: &mut Context<Self>) {
            self.status = status.into();
            cx.notify();
        }

        fn show_selection(&mut self, source: &str, path: Option<PathBuf>, cx: &mut Context<Self>) {
            let status = match path {
                Some(path) => format!("{source} : {}", path.display()),
                None => format!("{source} : aucun fichier choisi"),
            };
            self.set_status(status, cx);
        }

        fn browse_with_gpui(&mut self, cx: &mut Context<Self>) {
            let paths = cx.prompt_for_paths(PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: Some("Choisir".into()),
            });
            cx.spawn(async move |this, cx| {
                let path = match paths.await {
                    Ok(Ok(paths)) => paths.and_then(|paths| paths.into_iter().next()),
                    Ok(Err(error)) => {
                        let _ = this.update(cx, |this, cx| {
                            this.set_status(format!("GPUI : échec du sélecteur ({error})"), cx);
                        });
                        return;
                    }
                    Err(_) => None,
                };
                let _ = this.update(cx, |this, cx| this.show_selection("GPUI", path, cx));
            })
            .detach();
        }

        /// La boucle modale du sélecteur distribue les messages de la fenêtre propriétaire.
        /// Appelé depuis un écouteur GPUI, elle relancerait les tâches pendant que `App` est
        /// emprunté : le sélecteur tourne donc sur son propre thread, comme les dialogues GPUI.
        fn browse_with_win32(&mut self, window: &mut Window, cx: &mut Context<Self>) {
            let owner = window_hwnd(window).map(|hwnd| hwnd.0 as isize);
            let (sender, receiver) = futures::channel::oneshot::channel();
            let started = std::thread::Builder::new()
                .name("probe-file-dialog".into())
                .spawn(move || {
                    let owner = owner.map(|hwnd| HWND(hwnd as *mut _));
                    let _ = sender.send(choose_executable(owner));
                });
            if let Err(error) = started {
                self.set_status(format!("Win32 : thread impossible ({error})"), cx);
                return;
            }
            cx.spawn(async move |this, cx| {
                let status = receiver.await;
                let _ = this.update(cx, |this, cx| match status {
                    Ok(Ok(path)) => this.show_selection("Win32", path, cx),
                    Ok(Err(error)) => this.set_status(format!("Win32 : {error}"), cx),
                    Err(_) => this.set_status("Win32 : sélecteur interrompu", cx),
                });
            })
            .detach();
        }
    }

    impl Render for ProbeView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let following = self.following;
            let knob_offset = if following { px(22.) } else { px(2.) };
            div()
                .id("root")
                .role(Role::Application)
                .aria_label("Sonde GPUI")
                .track_focus(&self.focus_handle)
                .on_action(cx.listener(|_, _: &Tab, window, cx| window.focus_next(cx)))
                .on_action(cx.listener(|_, _: &TabPrev, window, cx| window.focus_prev(cx)))
                .size_full()
                .flex()
                .flex_col()
                .bg(rgb(0xf7f7f8))
                .text_color(rgb(0x1b1b1f))
                .font_family("Segoe UI")
                .child(
                    div()
                        .id("titlebar")
                        .h(px(36.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .pl_3()
                        .bg(rgb(0xe9e9ec))
                        .child(
                            div()
                                .id("title")
                                .flex_1()
                                .h_full()
                                .flex()
                                .items_center()
                                .window_control_area(WindowControlArea::Drag)
                                .child("Sonde GPUI"),
                        )
                        .child(
                            div()
                                .id("close")
                                .role(Role::Button)
                                .aria_label("Fermer la fenêtre")
                                .w(px(46.))
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .hover(|style| style.bg(rgb(0xc42b1c)).text_color(rgb(0xffffff)))
                                .window_control_area(WindowControlArea::Close)
                                .child("✕"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(probe_button("browse-gpui", "Parcourir avec GPUI").on_click(
                                    cx.listener(|this, _, _, cx| this.browse_with_gpui(cx)),
                                ))
                                .child(
                                    probe_button("browse-win32", "Parcourir avec Win32").on_click(
                                        cx.listener(|this, _, window, cx| {
                                            this.browse_with_win32(window, cx)
                                        }),
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .id("follow")
                                        .focusable()
                                        .tab_stop(true)
                                        .role(Role::Switch)
                                        .aria_label("Suivre")
                                        .aria_toggled(if following {
                                            Toggled::True
                                        } else {
                                            Toggled::False
                                        })
                                        .w(px(44.))
                                        .h(px(24.))
                                        .rounded_full()
                                        .cursor_pointer()
                                        .bg(if following {
                                            rgb(0x2f6feb)
                                        } else {
                                            rgb(0x8a8a93)
                                        })
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.following = !this.following;
                                            let state = if this.following {
                                                "activé"
                                            } else {
                                                "désactivé"
                                            };
                                            this.set_status(format!("Suivi {state}"), cx);
                                        }))
                                        .child(
                                            div()
                                                .size(px(20.))
                                                .mt(px(2.))
                                                .rounded_full()
                                                .bg(rgb(0xffffff))
                                                .with_spring(
                                                    "follow-knob",
                                                    SpringAnimation::new(SWITCH_SPRING)
                                                        .to(knob_offset)
                                                        .with_epsilon(0.25),
                                                    |knob, offset| knob.ml(offset),
                                                ),
                                        ),
                                )
                                .child("Suivre"),
                        )
                        .child(
                            div()
                                .id("software")
                                .role(Role::List)
                                .aria_label("Logiciels associés")
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(SOFTWARE.iter().enumerate().map(|(index, name)| {
                                    let selected = index == self.selected;
                                    div()
                                        .id(("software-item", index))
                                        .focusable()
                                        .tab_stop(true)
                                        .role(Role::ListItem)
                                        .aria_label(SharedString::from(format!(
                                            "{}. {name}",
                                            index + 1
                                        )))
                                        .aria_position_in_set(index + 1)
                                        .aria_size_of_set(SOFTWARE.len())
                                        .aria_selected(selected)
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .when(selected, |item| item.bg(rgb(0xdbe6fd)))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.selected = index;
                                            this.set_status(format!("{name} sélectionné"), cx);
                                        }))
                                        .child(format!("{}. {name}", index + 1))
                                })),
                        )
                        .child(
                            div()
                                .id("status")
                                .role(Role::Status)
                                .aria_label(self.status.clone())
                                .text_sm()
                                .child(self.status.clone()),
                        ),
                )
        }
    }

    fn probe_button(id: &'static str, label: &'static str) -> gpui::Stateful<gpui::Div> {
        div()
            .id(id)
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(label)
            .px_3()
            .py_1()
            .rounded_md()
            .bg(rgb(0x2f6feb))
            .text_color(rgb(0xffffff))
            .cursor_pointer()
            .child(label)
    }
}
