//! Captures hors écran de l'interface GPUI sous Linux (T057), comparées aux maquettes de
//! `specs/001-app-power-profiles/design/ecrans/`. Il faut un pilote Vulkan, par exemple
//! lavapipe (`mesa-vulkan-drivers`).
//!
//! `cargo run --features screenshots --bin screenshots -- <dossier>`

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("Les captures hors écran se font sous Linux.");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() -> anyhow::Result<()> {
    shots::run()
}

#[cfg(target_os = "linux")]
mod shots {

    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use gpui::{
        AnyWindowHandle, AppContext as _, Context, Entity, HeadlessAppContext, IntoElement,
        ParentElement, Render, Styled, Window, WindowAppearance, div, px, size,
    };
    use power_plan_sync::domain::PowerSource;
    use power_plan_sync::interface::assets::Assets;
    use power_plan_sync::interface::control_page::ControlPage;
    use power_plan_sync::interface::text_input::TextInput;
    use power_plan_sync::interface::theme::{self, ForcedAppearance, Theme};
    use power_plan_sync::interface::widgets::{
        self, Button, ButtonStyle, Message, MessageKind, Select, SelectOption, Switch,
    };
    use power_plan_sync::interface::window::{MainWindow, Page};
    use power_plan_sync::presentation::{Automation, Badge, UiDisplayModel};

    fn model(automation: Automation, power: PowerSource) -> UiDisplayModel {
        UiDisplayModel {
            automation,
            followed: "2 logiciels suivis sur 3".into(),
            power_source: match power {
                PowerSource::Ac => "Sur secteur",
                PowerSource::Battery => "Sur batterie",
                PowerSource::Unknown => "Alimentation inconnue",
            }
            .into(),
            power,
            ..UiDisplayModel::default()
        }
    }

    /// Les composants de T053, à la place de la page Logiciels qui arrive avec T058.
    struct Gallery {
        select: Entity<Select>,
        invalid: Entity<Select>,
        input: Entity<TextInput>,
    }

    impl Gallery {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            let options = || {
                [
                    "Utilisation normale",
                    "Haute performance",
                    "Performances optimales",
                ]
                .map(|label| SelectOption {
                    label: label.into(),
                })
                .to_vec()
            };
            let select = cx.new(|cx| {
                let mut select = Select::new("Plan pendant l’exécution", window, cx);
                select.set_options(options(), Some(1), cx);
                select
            });
            let invalid = cx.new(|cx| {
                let mut select = Select::new("Plan de retour", window, cx);
                select.set_options(options(), None, cx);
                select.error = Some("Ce plan n’existe plus dans Windows.".into());
                select
            });
            let input = cx.new(|cx| {
                let mut input = TextInput::new("Nom affiché", "Nom du logiciel", None, cx);
                input.set_text("Blender", cx);
                input
            });
            Self {
                select,
                invalid,
                input,
            }
        }
    }

    impl Render for Gallery {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let theme = Theme::of(window, cx);
            let row = || div().flex().flex_wrap().items_center().gap(px(12.));
            div()
                .p(px(24.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(
                    row()
                        .child(
                            Button::new("add", "Ajouter un logiciel", ButtonStyle::Tinted)
                                .icon("plus")
                                .keys(["Ctrl", "N"])
                                .backdrop(theme.editor),
                        )
                        .child(
                            Button::new("pause", "Mettre en pause", ButtonStyle::Subtle)
                                .icon("pause")
                                .keys(["Ctrl", "Alt", "P"])
                                .backdrop(theme.editor),
                        )
                        .child(Button::new("browse", "Parcourir…", ButtonStyle::Filled))
                        .child(Button::new("confirm", "Confirmer", ButtonStyle::Outlined))
                        .child(
                            Button::new("off", "Activer l’automatisation", ButtonStyle::Tinted)
                                .disabled(true),
                        )
                        .child(Button::icon_only("edit", "pencil", "Modifier")),
                )
                .child(
                    row()
                        .child(Switch::new("on", "Suivre Blender", true))
                        .child(Switch::new("off-switch", "Suivre Hollow Bloom", false))
                        .child(widgets::badge(Badge::Applied, &theme))
                        .child(widgets::badge(Badge::Open, &theme))
                        .child(widgets::badge(Badge::Suspended, &theme))
                        .child(widgets::badge(Badge::Refused, &theme))
                        .child(widgets::keys(["Ctrl", "Z"], &theme)),
                )
                .child(row().child(self.select.clone()).child(self.invalid.clone()))
                .child(div().w(px(280.)).child(self.input.clone()))
                .child(
                    Message::new(
                        "error",
                        MessageKind::Error,
                        "Une autre application a peut-être verrouillé les plans.",
                    )
                    .title("Windows a refusé le changement de plan")
                    .action(
                        Button::new("retry", "Réessayer", ButtonStyle::Outlined)
                            .icon("refresh-cw")
                            .backdrop(theme.error_background),
                    ),
                )
                .child(Message::new(
                    "info",
                    MessageKind::Info,
                    "Hollow Bloom a été retiré de la liste.",
                ))
                .child(widgets::field_error(
                    "field-error",
                    "Ce plan n’existe plus dans Windows.",
                    &theme,
                ))
        }
    }

    fn capture(
        cx: &mut HeadlessAppContext,
        window: AnyWindowHandle,
        path: &Path,
    ) -> anyhow::Result<()> {
        cx.run_until_parked();
        cx.update_window(window, |_, window, _| window.refresh())?;
        cx.run_until_parked();
        cx.capture_screenshot(window)?.save(path)?;
        println!("{}", path.display());
        Ok(())
    }

    pub fn run() -> anyhow::Result<()> {
        let output = PathBuf::from(std::env::args().nth(1).unwrap_or("out".into()));
        std::fs::create_dir_all(&output)?;
        let text_system = Arc::new(gpui_wgpu::CosmicTextSystem::new_without_system_fonts(
            theme::FONT_FAMILY,
        ));
        let mut cx = HeadlessAppContext::with_platform(
            text_system,
            Arc::new(Assets),
            gpui_platform::current_headless_renderer,
        );
        cx.update(|cx| {
            theme::load_fonts(cx)?;
            widgets::bind_keys(cx);
            power_plan_sync::interface::window::bind_keys(cx);
            power_plan_sync::interface::text_input::bind_keys(cx);
            anyhow::Ok(())
        })?;
        let cases = [
            (
                "dark",
                WindowAppearance::Dark,
                880.,
                620.,
                Automation::Active,
                PowerSource::Ac,
                Page::Software,
            ),
            (
                "light",
                WindowAppearance::Light,
                880.,
                620.,
                Automation::Active,
                PowerSource::Ac,
                Page::Software,
            ),
            (
                "narrow",
                WindowAppearance::Dark,
                420.,
                760.,
                Automation::Active,
                PowerSource::Ac,
                Page::Software,
            ),
            (
                "battery",
                WindowAppearance::Dark,
                880.,
                620.,
                Automation::Waiting,
                PowerSource::Battery,
                Page::Settings,
            ),
        ];
        for (name, appearance, width, height, automation, power, page) in cases {
            cx.update(|cx| cx.set_global(ForcedAppearance(appearance)));
            let window = cx.open_window(size(px(width), px(height)), |window, cx| {
                let gallery = cx.new(|cx| Gallery::new(window, cx));
                cx.new(|cx| {
                    let mut main = MainWindow::new(model(automation, power), window, cx);
                    main.set_page_view(Page::Software, gallery.into(), cx);
                    main.show_page(page, cx);
                    main
                })
            })?;
            capture(
                &mut cx,
                window.into(),
                &output.join(format!("window-{name}.png")),
            )?;
        }
        for (name, appearance) in [
            ("dark", WindowAppearance::Dark),
            ("light", WindowAppearance::Light),
        ] {
            cx.update(|cx| cx.set_global(ForcedAppearance(appearance)));
            // Comme le coordinateur sans contrôle : vue vide, plans et alimentation lus.
            let view = power_plan_sync::domain::ViewState {
                configuration: Default::default(),
                plans: vec![],
                observed_plan: None,
                status: power_plan_sync::domain::Status::Blocked(
                    power_plan_sync::domain::Error::InvalidConfiguration(
                        power_plan_sync::interface::strings::CONTROL_HELD.into(),
                    ),
                ),
                cycle: None,
                power_source: PowerSource::Ac,
                open_rules: vec![],
                poll_processes: false,
                can_exit: true,
            };
            let mut unavailable = power_plan_sync::presentation::display(&view, false, false, None);
            unavailable.read_only = true;
            let window = cx.open_window(size(px(880.), px(620.)), |window, cx| {
                let page = cx.new(|_| ControlPage::new(unavailable.clone(), |_| {}));
                cx.new(|cx| {
                    let mut main = MainWindow::new(unavailable.clone(), window, cx);
                    main.set_page_view(Page::Software, page.into(), cx);
                    main
                })
            })?;
            capture(
                &mut cx,
                window.into(),
                &output.join(format!("unavailable-{name}.png")),
            )?;
        }
        Ok(())
    }
}
