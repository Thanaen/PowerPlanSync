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
    use power_plan_sync::domain::{
        AppRule, Configuration, Cycle, Error, ExecutablePath, PlanId, PowerPlan, RuleId, Status,
        ViewState,
    };
    use power_plan_sync::interface::assets::Assets;
    use power_plan_sync::interface::rule_dialog::{RuleDialog, RuleDialogMode};
    use power_plan_sync::interface::software_page::SoftwarePage;
    use power_plan_sync::interface::strings;
    use power_plan_sync::interface::text_input::TextInput;
    use power_plan_sync::interface::theme::{self, ForcedAppearance, Theme};
    use power_plan_sync::interface::widgets::{
        self, Button, ButtonStyle, Message, MessageKind, Select, SelectOption, Switch,
    };
    use power_plan_sync::interface::window::{MainWindow, Page};
    use power_plan_sync::presentation::{
        Badge, RunningAppDisplay, UiDisplayModel, after_failed_quit, display, settings_unreadable,
        without_control,
    };

    fn rule(id: u64, label: &str, path: &str, plan: u128, enabled: bool) -> AppRule {
        AppRule {
            id: RuleId(id),
            label: label.into(),
            executable: ExecutablePath(path.encode_utf16().collect()),
            plan: PlanId(plan),
            enabled,
        }
    }

    /// L'état des maquettes : trois logiciels, Battlefield 6 ouvert sur secteur.
    fn active() -> ViewState {
        ViewState {
            configuration: Configuration {
                rules: vec![
                    rule(
                        1,
                        "Battlefield 6",
                        r"C:\Jeux\EA\Battlefield 6\bf6.exe",
                        2,
                        true,
                    ),
                    rule(
                        2,
                        "Blender",
                        r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe",
                        3,
                        true,
                    ),
                    rule(
                        3,
                        "Hollow Bloom",
                        r"D:\Jeux\Steam\Hollow Bloom\HollowBloom.exe",
                        2,
                        false,
                    ),
                ],
                return_plan: Some(PlanId(1)),
                automation_enabled: true,
                ..Default::default()
            },
            plans: [
                (1, "Utilisation normale"),
                (2, "Haute performance"),
                (3, "Performances optimales"),
            ]
            .map(|(id, name)| PowerPlan {
                id: PlanId(id),
                name: name.into(),
            })
            .to_vec(),
            observed_plan: Some(PlanId(2)),
            status: Status::Active,
            cycle: Some(Cycle {
                id: 1,
                initial: PlanId(1),
                return_plan: PlanId(1),
                last_confirmed: PlanId(2),
                winner: RuleId(1),
                acquired: true,
            }),
            power_source: PowerSource::Ac,
            open_rules: vec![RuleId(1), RuleId(2)],
            poll_processes: true,
            can_exit: false,
        }
    }

    fn paused() -> ViewState {
        let mut view = active();
        view.configuration.paused = true;
        view.status = Status::Paused;
        view.cycle = None;
        view.observed_plan = Some(PlanId(1));
        view
    }

    fn battery() -> ViewState {
        let mut view = active();
        view.status = Status::Idle;
        view.cycle = None;
        view.observed_plan = Some(PlanId(1));
        view.power_source = PowerSource::Battery;
        view.open_rules = vec![RuleId(1)];
        view
    }

    /// **Plan modifié dans Windows** : Économies d'énergie choisi pendant le cycle de
    /// Battlefield 6.
    fn external() -> ViewState {
        let mut view = active();
        view.plans.push(PowerPlan {
            id: PlanId(4),
            name: "Économies d’énergie".into(),
        });
        view.observed_plan = Some(PlanId(4));
        view.status = Status::ManualOverride;
        view
    }

    /// Windows garde Utilisation normale au lieu de Haute performance pour Battlefield 6.
    fn refused() -> ViewState {
        let mut view = active();
        view.observed_plan = Some(PlanId(1));
        view.status = Status::Blocked(Error::Power("refus".into()));
        if let Some(cycle) = &mut view.cycle {
            cycle.last_confirmed = PlanId(1);
        }
        view
    }

    /// Le retour à Utilisation normale a échoué en quittant : Haute performance reste actif.
    fn quit_failed() -> UiDisplayModel {
        let mut view = active();
        view.status = Status::Blocked(Error::Power("refus".into()));
        after_failed_quit(display(&view, false, false, None), &view)
    }

    /// Blender vient d'être retiré : **Annuler** peut le remettre.
    fn removed() -> UiDisplayModel {
        let mut view = active();
        view.configuration.rules.remove(1);
        let mut model = display(&view, false, false, None);
        model.removed = Some("Blender".into());
        model
    }

    /// `config.json` illisible au lancement : sans contrôle ni liste.
    fn unreadable() -> UiDisplayModel {
        let view = ViewState {
            configuration: Default::default(),
            observed_plan: Some(PlanId(1)),
            status: Status::Blocked(Error::InvalidConfiguration("illisible".into())),
            cycle: None,
            open_rules: vec![],
            poll_processes: false,
            can_exit: true,
            ..active()
        };
        settings_unreadable(display(&view, false, false, None))
    }

    /// « Pour commencer » : rien d'ajouté, avec ou sans le plan normal, puis prêt à activer.
    fn first_launch(normal_plan: bool, ready: bool) -> ViewState {
        let normal = PlanId(0x381b4222_f694_41f0_9685_ff5bb260df2e);
        let mut view = active();
        let first = if normal_plan {
            (normal, "Utilisation normale")
        } else {
            (PlanId(4), "Équilibré")
        };
        view.plans = [
            first,
            (PlanId(2), "Haute performance"),
            (PlanId(3), "Performances optimales"),
        ]
        .map(|(id, name)| PowerPlan {
            id,
            name: name.into(),
        })
        .to_vec();
        view.observed_plan = Some(first.0);
        view.configuration.automation_enabled = false;
        view.configuration.return_plan = ready.then_some(normal);
        view.configuration.rules.truncate(if ready { 1 } else { 0 });
        view.status = if ready {
            Status::Idle
        } else {
            Status::Unconfigured
        };
        view.cycle = None;
        view.open_rules.clear();
        view
    }

    /// Comme le coordinateur sans contrôle : configuration non lue, plan actif lu, refus en
    /// erreur.
    fn unavailable() -> UiDisplayModel {
        let view = ViewState {
            configuration: Default::default(),
            observed_plan: Some(PlanId(2)),
            status: Status::Blocked(Error::InvalidConfiguration(strings::CONTROL_HELD.into())),
            cycle: None,
            open_rules: vec![],
            poll_processes: false,
            can_exit: true,
            ..active()
        };
        without_control(display(&view, false, false, None))
    }

    fn running(label: &str, path: &str) -> RunningAppDisplay {
        RunningAppDisplay {
            label: label.into(),
            path: path.into(),
            accessible: !path.is_empty(),
        }
    }

    /// Les dialogues de T059 sur la page **Logiciels**, dans l'état de leur maquette.
    #[derive(Clone, Copy)]
    enum DialogCase {
        /// **Application ouverte** : Battlefield 6 choisi, une application inaccessible.
        Add,
        /// Rien de choisi puis Entrée : chaque champ dit quoi corriger.
        AddInvalid,
        /// Hollow Bloom introuvable.
        EditMissing,
    }

    fn dialog(
        case: DialogCase,
        model: &UiDisplayModel,
        window: &mut Window,
        cx: &mut Context<RuleDialog>,
    ) -> RuleDialog {
        let mode = match case {
            DialogCase::Add | DialogCase::AddInvalid => RuleDialogMode::Add,
            DialogCase::EditMissing => RuleDialogMode::Edit {
                rule: model.rules[2].clone(),
                missing: true,
            },
        };
        let mut dialog = RuleDialog::new(mode, model, false, |_, _| {}, window, cx);
        match case {
            DialogCase::Add => {
                let bf6 = r"C:\Jeux\EA\Battlefield 6\bf6.exe";
                dialog.set_running_apps(
                    vec![
                        running("bf6.exe", bf6),
                        running(
                            "EADesktop.exe",
                            r"C:\Program Files\Electronic Arts\EA Desktop\EADesktop.exe",
                        ),
                        running("AntiCheat.exe", ""),
                    ],
                    cx,
                );
                dialog.choose(0, cx);
                dialog.set_executable(bf6.into(), "Battlefield 6".into(), cx);
                dialog.select_plan(1, cx);
            }
            DialogCase::AddInvalid => {
                dialog.set_running_apps(Vec::new(), cx);
                dialog.confirm(window, cx);
            }
            DialogCase::EditMissing => {}
        }
        dialog
    }

    /// Les composants de T053, rendus seuls.
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
                .size_full()
                .bg(theme.editor)
                .font_family(theme::FONT_FAMILY)
                .text_color(theme.text)
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
            power_plan_sync::interface::rule_dialog::bind_keys(cx);
            anyhow::Ok(())
        })?;
        let cases = [
            (
                "main-dark",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&active(), false, false, None),
            ),
            (
                "main-light",
                WindowAppearance::Light,
                880.,
                620.,
                display(&active(), false, false, None),
            ),
            (
                "main-narrow",
                WindowAppearance::Dark,
                420.,
                760.,
                display(&active(), false, false, None),
            ),
            (
                "paused",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&paused(), false, false, None),
            ),
            (
                "battery",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&battery(), false, false, None),
            ),
            (
                "first-launch",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&first_launch(true, false), false, false, None),
            ),
            (
                "first-launch-missing",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&first_launch(false, false), false, false, None),
            ),
            (
                "first-launch-ready",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&first_launch(true, true), false, false, None),
            ),
            (
                "first-launch-unfollowed",
                WindowAppearance::Dark,
                880.,
                620.,
                display(
                    &{
                        let mut view = first_launch(true, true);
                        view.configuration.rules[0].enabled = false;
                        view
                    },
                    false,
                    false,
                    None,
                ),
            ),
            (
                "first-launch-light",
                WindowAppearance::Light,
                880.,
                620.,
                display(&first_launch(true, false), false, false, None),
            ),
            (
                "first-launch-narrow",
                WindowAppearance::Dark,
                420.,
                760.,
                display(&first_launch(true, false), false, false, None),
            ),
            (
                "external",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&external(), false, false, None),
            ),
            (
                "refused",
                WindowAppearance::Dark,
                880.,
                620.,
                display(&refused(), false, false, None),
            ),
            (
                "refused-light",
                WindowAppearance::Light,
                880.,
                620.,
                display(&refused(), false, false, None),
            ),
            (
                "quit-failed",
                WindowAppearance::Dark,
                880.,
                620.,
                quit_failed(),
            ),
            (
                "quit-failed-narrow",
                WindowAppearance::Dark,
                420.,
                760.,
                quit_failed(),
            ),
            ("removed", WindowAppearance::Dark, 880., 620., removed()),
            (
                "removed-light",
                WindowAppearance::Light,
                880.,
                620.,
                removed(),
            ),
            (
                "removed-narrow",
                WindowAppearance::Dark,
                420.,
                760.,
                removed(),
            ),
            (
                "settings-unreadable",
                WindowAppearance::Dark,
                880.,
                620.,
                unreadable(),
            ),
            (
                "settings-unreadable-light",
                WindowAppearance::Light,
                880.,
                620.,
                unreadable(),
            ),
            (
                "unavailable-dark",
                WindowAppearance::Dark,
                880.,
                620.,
                unavailable(),
            ),
            (
                "unavailable-light",
                WindowAppearance::Light,
                880.,
                620.,
                unavailable(),
            ),
        ];
        for (name, appearance, width, height, model) in cases {
            cx.update(|cx| cx.set_global(ForcedAppearance(appearance)));
            let window = cx.open_window(size(px(width), px(height)), |window, cx| {
                let page = cx.new(|_| SoftwarePage::new(model.clone(), |_, _| {}));
                cx.new(|cx| {
                    let mut main = MainWindow::new(model.clone(), window, cx);
                    main.set_page_view(Page::Software, page.into(), cx);
                    main
                })
            })?;
            capture(&mut cx, window.into(), &output.join(format!("{name}.png")))?;
        }
        let empty = {
            let mut view = active();
            view.configuration.rules.clear();
            view.configuration.automation_enabled = false;
            view.configuration.return_plan = None;
            view.status = Status::Unconfigured;
            view.cycle = None;
            view.open_rules.clear();
            view.observed_plan = Some(PlanId(1));
            display(&view, false, false, None)
        };
        let dialogs = [
            (
                "add-rule",
                WindowAppearance::Dark,
                DialogCase::Add,
                empty.clone(),
            ),
            (
                "add-rule-light",
                WindowAppearance::Light,
                DialogCase::Add,
                empty.clone(),
            ),
            (
                "add-rule-invalid",
                WindowAppearance::Dark,
                DialogCase::AddInvalid,
                empty,
            ),
            (
                "edit-rule",
                WindowAppearance::Dark,
                DialogCase::EditMissing,
                display(&active(), false, false, None),
            ),
        ];
        for (name, appearance, case, model) in dialogs {
            cx.update(|cx| cx.set_global(ForcedAppearance(appearance)));
            let window = cx.open_window(size(px(880.), px(620.)), |window, cx| {
                let page = cx.new(|_| SoftwarePage::new(model.clone(), |_, _| {}));
                let dialog = cx.new(|cx| dialog(case, &model, window, cx));
                cx.new(|cx| {
                    let mut main = MainWindow::new(model.clone(), window, cx);
                    main.set_page_view(Page::Software, page.into(), cx);
                    main.set_dialog(Some(dialog.into()), cx);
                    main
                })
            })?;
            capture(&mut cx, window.into(), &output.join(format!("{name}.png")))?;
        }
        for (name, appearance) in [
            ("widgets-dark", WindowAppearance::Dark),
            ("widgets-light", WindowAppearance::Light),
        ] {
            cx.update(|cx| cx.set_global(ForcedAppearance(appearance)));
            let window = cx.open_window(size(px(880.), px(520.)), |window, cx| {
                cx.new(|cx| Gallery::new(window, cx))
            })?;
            capture(&mut cx, window.into(), &output.join(format!("{name}.png")))?;
        }
        Ok(())
    }
}
