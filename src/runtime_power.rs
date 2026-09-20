use crate::{coordinator::PowerPort, domain::*};

pub enum RuntimePower {
    Live,
    Demo { active: PlanId },
}
impl RuntimePower {
    pub fn new(demo: bool) -> Self {
        if demo {
            Self::Demo { active: PlanId(1) }
        } else {
            Self::Live
        }
    }
}
impl PowerPort for RuntimePower {
    fn active_plan(&mut self) -> Result<PlanId, Error> {
        match self {
            Self::Demo { active } => Ok(*active),
            Self::Live => crate::windows::power::active_plan()
                .map(|id| PlanId(id.as_guid().to_u128()))
                .map_err(|e| Error::Power(e.to_string())),
        }
    }
    fn set_active_plan(&mut self, plan: PlanId) -> Result<(), Error> {
        match self {
            Self::Demo { active } => {
                *active = plan;
                Ok(())
            }
            Self::Live => crate::windows::power::set_active_plan(
                crate::windows::power::PlanId::from_guid(windows::core::GUID::from_u128(plan.0)),
            )
            .map(|_| ())
            .map_err(|e| Error::Power(e.to_string())),
        }
    }
    fn plans(&mut self) -> Result<Vec<PowerPlan>, Error> {
        match self {
            Self::Demo { .. } => Ok(vec![
                PowerPlan {
                    id: PlanId(1),
                    name: "Équilibré (simulation)".into(),
                },
                PowerPlan {
                    id: PlanId(2),
                    name: "Haute performance (simulation)".into(),
                },
            ]),
            Self::Live => crate::windows::power::enumerate_plans()
                .map(|plans| {
                    plans
                        .into_iter()
                        .map(|p| PowerPlan {
                            id: PlanId(p.id.as_guid().to_u128()),
                            name: p.name,
                        })
                        .collect()
                })
                .map_err(|e| Error::Power(e.to_string())),
        }
    }
}

pub fn error_message(error: &Error) -> String {
    match error {
        Error::Power(_) => "Windows n'a pas confirmé le plan demandé. Vérifiez les réglages puis réessayez.".into(),
        Error::Storage(_) => "La configuration ou le journal ne peut pas être lu ou enregistré. Les fichiers sont conservés. Vérifiez le dossier de données puis réessayez.".into(),
        Error::MissingPlan(_) => "Le plan demandé n'est plus disponible. Choisissez un plan installé.".into(),
        Error::InvalidConfiguration(message) => message.clone(),
        Error::ObservationUncertain => "Ce logiciel ne peut pas être identifié. L'automatisation est suspendue.".into(),
        Error::ForeignOwner => "Le journal appartient à un autre utilisateur. Aucun plan n'a été modifié.".into(),
        Error::StaleTransaction => "Le journal a changé. L'automatisation est suspendue pour conserver la récupération.".into(),
    }
}
