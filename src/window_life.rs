//! Cycle de vie du processus de fenêtre vu par le processus de suivi (D18), sans Windows : la
//! table d'états de `contracts/runtime.md` et la file des commandes en attente. Le processus de
//! suivi applique les effets rendus ; il écarte avant elle les événements d'une instance qu'il
//! a déjà arrêtée ou remplacée.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::presentation::UiCommand;

/// Délai de la première réponse d'un processus de fenêtre lancé.
pub const LAUNCH_TIMEOUT: Duration = Duration::from_secs(15);
/// Délai de la réponse à `Open`, ou à une nouvelle erreur, d'une fenêtre affichée.
pub const REOPEN_TIMEOUT: Duration = Duration::from_secs(5);
/// Commandes autres que `Update` en attente au-delà desquelles la fenêtre ne lit plus.
pub const MAX_PENDING: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Absent,
    /// Lancé, `WindowOpened` attendu avant `deadline`.
    Launching {
        deadline: Instant,
    },
    Shown,
    /// `Open` ou une nouvelle erreur envoyé, `WindowOpened` attendu avant `deadline`.
    Reopening {
        deadline: Instant,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Clic sur l'icône, **Ouvrir PowerPlanSync** ou second lancement.
    OpenRequested,
    /// Une nouvelle erreur, déjà envoyée à la fenêtre avec son modèle.
    Alert,
    /// `UiAction::WindowOpened`.
    Opened,
    /// Fin du canal des actions ; `clean` pour un code de sortie nul.
    Ended { clean: bool },
    /// L'horloge a passé `now`.
    Tick,
    /// La file des commandes est pleine.
    Saturated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loss {
    /// `WindowOpened` n'est pas arrivé à temps.
    NoAnswer,
    /// Arrêt avec un autre code, ou trame refusée.
    Crashed,
    /// Plus de place dans la file des commandes.
    Saturated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Launch,
    SendOpen,
    Terminate,
    /// Au premier masquage, « PowerPlanSync reste ouvert ».
    ExplainHidden,
    /// Noter la perte au journal ; `notify` l'explique aussi par l'icône.
    Lost {
        loss: Loss,
        notify: bool,
    },
}

/// Applique `input` à `phase` à l'instant `now`, et rend la nouvelle phase et ses effets, dans
/// l'ordre où les appliquer.
pub fn step(phase: Phase, input: Input, now: Instant) -> (Phase, Vec<Effect>) {
    use Effect::*;
    let launch = Phase::Launching {
        deadline: now + LAUNCH_TIMEOUT,
    };
    let lost = |loss, notify| Lost { loss, notify };
    match (phase, input) {
        (Phase::Absent, Input::OpenRequested | Input::Alert) => (launch, vec![Launch]),
        (Phase::Absent, _) => (phase, vec![]),
        (Phase::Launching { .. } | Phase::Reopening { .. }, Input::Opened) => {
            (Phase::Shown, vec![])
        }
        (Phase::Shown, Input::Opened) => (phase, vec![]),
        (_, Input::Saturated) => (Phase::Absent, vec![Terminate, lost(Loss::Saturated, true)]),
        (Phase::Shown, Input::OpenRequested) => (reopen(now), vec![SendOpen]),
        (Phase::Shown, Input::Alert) => (reopen(now), vec![]),
        (Phase::Shown, Input::Ended { clean: true }) => (Phase::Absent, vec![ExplainHidden]),
        (Phase::Launching { .. } | Phase::Shown, Input::Ended { .. }) => {
            (Phase::Absent, vec![lost(Loss::Crashed, true)])
        }
        (Phase::Reopening { .. }, Input::Ended { clean: true }) => (launch, vec![Launch]),
        (Phase::Reopening { .. }, Input::Ended { clean: false }) => {
            (launch, vec![lost(Loss::Crashed, false), Launch])
        }
        (Phase::Launching { deadline }, Input::Tick) if now >= deadline => {
            (Phase::Absent, vec![Terminate, lost(Loss::NoAnswer, true)])
        }
        (Phase::Reopening { deadline }, Input::Tick) if now >= deadline => {
            (launch, vec![Terminate, lost(Loss::NoAnswer, false), Launch])
        }
        (_, Input::Tick | Input::OpenRequested | Input::Alert) => (phase, vec![]),
    }
}

fn reopen(now: Instant) -> Phase {
    Phase::Reopening {
        deadline: now + REOPEN_TIMEOUT,
    }
}

/// L'échéance que l'horloge doit surveiller, s'il y en a une.
pub fn deadline(phase: Phase) -> Option<Instant> {
    match phase {
        Phase::Launching { deadline } | Phase::Reopening { deadline } => Some(deadline),
        Phase::Absent | Phase::Shown => None,
    }
}

/// Les commandes en attente vers la fenêtre. Un nouveau modèle remplace l'ancien : seul le
/// dernier `Update` attend.
#[derive(Default)]
pub struct Outbox {
    commands: VecDeque<UiCommand>,
}

impl Outbox {
    /// Rend faux si la file est pleine : la fenêtre ne lit plus.
    pub fn push(&mut self, command: UiCommand) -> bool {
        if let UiCommand::Update { model, alert } = command {
            // Le modèle remplacé gardait peut-être une alerte : elle passe au suivant.
            let mut kept_alert = alert;
            self.commands.retain(|pending| match pending {
                UiCommand::Update { alert, .. } => {
                    kept_alert |= *alert;
                    false
                }
                _ => true,
            });
            self.commands.push_back(UiCommand::Update {
                model,
                alert: kept_alert,
            });
            return true;
        }
        let others = self
            .commands
            .iter()
            .filter(|pending| !matches!(pending, UiCommand::Update { .. }))
            .count();
        if others >= MAX_PENDING {
            return false;
        }
        self.commands.push_back(command);
        true
    }

    pub fn pop(&mut self) -> Option<UiCommand> {
        self.commands.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::UiDisplayModel;

    fn at(seconds: u64) -> Instant {
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        *START.get_or_init(Instant::now) + Duration::from_secs(seconds)
    }

    fn launching(since: u64) -> Phase {
        Phase::Launching {
            deadline: at(since) + LAUNCH_TIMEOUT,
        }
    }

    fn reopening(since: u64) -> Phase {
        Phase::Reopening {
            deadline: at(since) + REOPEN_TIMEOUT,
        }
    }

    fn lost(loss: Loss, notify: bool) -> Effect {
        Effect::Lost { loss, notify }
    }

    /// Une ligne par ligne de la table d'états de `contracts/runtime.md`, sauf `Quit` du
    /// coordinateur : le processus de suivi ferme alors la fenêtre lui-même, hors de cette table.
    #[test]
    fn every_row_of_the_state_table() {
        use Effect::*;
        use Input::*;
        let rows: Vec<(Phase, Input, u64, Phase, Vec<Effect>)> = vec![
            (Phase::Absent, OpenRequested, 0, launching(0), vec![Launch]),
            (Phase::Absent, Alert, 0, launching(0), vec![Launch]),
            (launching(0), OpenRequested, 3, launching(0), vec![]),
            (launching(0), Alert, 3, launching(0), vec![]),
            (launching(0), Opened, 3, Phase::Shown, vec![]),
            (
                launching(0),
                Tick,
                16,
                Phase::Absent,
                vec![Terminate, lost(Loss::NoAnswer, true)],
            ),
            (
                launching(0),
                Ended { clean: true },
                2,
                Phase::Absent,
                vec![lost(Loss::Crashed, true)],
            ),
            (
                launching(0),
                Ended { clean: false },
                2,
                Phase::Absent,
                vec![lost(Loss::Crashed, true)],
            ),
            (
                Phase::Shown,
                OpenRequested,
                30,
                reopening(30),
                vec![SendOpen],
            ),
            (Phase::Shown, Alert, 30, reopening(30), vec![]),
            (
                Phase::Shown,
                Ended { clean: true },
                30,
                Phase::Absent,
                vec![ExplainHidden],
            ),
            (
                Phase::Shown,
                Ended { clean: false },
                30,
                Phase::Absent,
                vec![lost(Loss::Crashed, true)],
            ),
            (reopening(30), OpenRequested, 32, reopening(30), vec![]),
            (reopening(30), Alert, 32, reopening(30), vec![]),
            (reopening(30), Opened, 32, Phase::Shown, vec![]),
            (
                reopening(30),
                Tick,
                36,
                launching(36),
                vec![Terminate, lost(Loss::NoAnswer, false), Launch],
            ),
            (
                reopening(30),
                Ended { clean: true },
                31,
                launching(31),
                vec![Launch],
            ),
            (
                reopening(30),
                Ended { clean: false },
                31,
                launching(31),
                vec![lost(Loss::Crashed, false), Launch],
            ),
        ];
        for (phase, input, now, expected_phase, expected_effects) in rows {
            let (next, effects) = step(phase, input, at(now));
            assert_eq!(next, expected_phase, "{phase:?} + {input:?}");
            assert_eq!(effects, expected_effects, "{phase:?} + {input:?}");
        }
    }

    #[test]
    fn a_tick_before_the_deadline_changes_nothing() {
        assert_eq!(
            step(launching(0), Input::Tick, at(14)),
            (launching(0), vec![])
        );
        assert_eq!(
            step(reopening(30), Input::Tick, at(34)),
            (reopening(30), vec![])
        );
        assert_eq!(
            step(Phase::Shown, Input::Tick, at(99)),
            (Phase::Shown, vec![])
        );
        assert_eq!(
            step(Phase::Absent, Input::Tick, at(99)),
            (Phase::Absent, vec![])
        );
    }

    #[test]
    fn a_relaunch_that_never_answers_is_not_relaunched_again() {
        let (relaunched, effects) = step(reopening(30), Input::Tick, at(36));
        assert!(effects.contains(&Effect::Launch));
        let (next, effects) = step(relaunched, Input::Tick, at(36 + 16));
        assert_eq!(next, Phase::Absent);
        assert!(!effects.contains(&Effect::Launch));
    }

    #[test]
    fn a_saturated_window_is_stopped_and_reported() {
        for phase in [launching(0), Phase::Shown, reopening(30)] {
            assert_eq!(
                step(phase, Input::Saturated, at(40)),
                (
                    Phase::Absent,
                    vec![Effect::Terminate, lost(Loss::Saturated, true)]
                ),
                "{phase:?}"
            );
        }
    }

    #[test]
    fn events_of_an_absent_window_change_nothing() {
        for input in [
            Input::Opened,
            Input::Ended { clean: false },
            Input::Saturated,
        ] {
            assert_eq!(
                step(Phase::Absent, input, at(1)),
                (Phase::Absent, vec![]),
                "{input:?}"
            );
        }
    }

    #[test]
    fn only_waiting_phases_have_a_deadline() {
        assert_eq!(deadline(launching(0)), Some(at(15)));
        assert_eq!(deadline(reopening(30)), Some(at(35)));
        assert_eq!(deadline(Phase::Shown), None);
        assert_eq!(deadline(Phase::Absent), None);
    }

    fn update(status: &str) -> UiCommand {
        UiCommand::Update {
            model: Box::new(UiDisplayModel {
                status: status.into(),
                ..Default::default()
            }),
            alert: false,
        }
    }

    #[test]
    fn only_the_latest_model_waits_and_order_is_kept() {
        let mut outbox = Outbox::default();
        assert!(outbox.push(update("ancien")));
        assert!(outbox.push(UiCommand::Open));
        assert!(outbox.push(update("nouveau")));
        assert_eq!(outbox.pop(), Some(UiCommand::Open));
        assert_eq!(outbox.pop(), Some(update("nouveau")));
        assert_eq!(outbox.pop(), None);
    }

    #[test]
    fn an_alert_is_never_merged_away() {
        let mut outbox = Outbox::default();
        let alert = UiCommand::Update {
            model: Box::new(UiDisplayModel::default()),
            alert: true,
        };
        assert!(outbox.push(alert));
        assert!(outbox.push(update("après")));
        let UiCommand::Update { alert, .. } = outbox.pop().unwrap() else {
            panic!()
        };
        assert!(alert, "le dernier modèle garde l'alerte qu'il remplace");
    }

    #[test]
    fn the_outbox_refuses_beyond_the_limit_whatever_the_models() {
        let mut outbox = Outbox::default();
        for _ in 0..MAX_PENDING {
            assert!(outbox.push(UiCommand::PreferencesChanged));
            assert!(outbox.push(update("modèle")));
        }
        assert!(!outbox.push(UiCommand::PreferencesChanged));
        assert!(outbox.push(update("toujours remplacé")));
    }
}
