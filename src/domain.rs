use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlanId(pub u128);
impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PowerPlan {
    pub id: PlanId,
    pub name: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutablePath(pub Vec<u16>);
impl ExecutablePath {
    pub fn display(&self) -> String {
        String::from_utf16_lossy(&self.0)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RuleId(pub u64);
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppRule {
    pub id: RuleId,
    pub label: String,
    pub executable: ExecutablePath,
    pub plan: PlanId,
    pub enabled: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    pub schema_version: u32,
    pub rules: Vec<AppRule>,
    pub return_plan: Option<PlanId>,
    pub automation_enabled: bool,
    pub paused: bool,
    pub allow_on_battery: bool,
    pub start_at_login: bool,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            schema_version: 1,
            rules: vec![],
            return_plan: None,
            automation_enabled: false,
            paused: false,
            allow_on_battery: false,
            start_at_login: false,
        }
    }
}
impl Configuration {
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != 1 {
            return Err(Error::Storage(
                "Version de configuration incompatible".into(),
            ));
        }
        if self.automation_enabled && !self.rules.iter().any(|r| r.enabled) {
            return Err(Error::InvalidConfiguration(
                "Ajoutez une règle active".into(),
            ));
        }
        if self.automation_enabled && self.return_plan.is_none() {
            return Err(Error::InvalidConfiguration(
                "Choisissez un plan de retour".into(),
            ));
        }
        for (i, rule) in self.rules.iter().enumerate() {
            if rule.executable.0.is_empty() || rule.executable.0.contains(&0) {
                return Err(Error::InvalidConfiguration("Chemin invalide".into()));
            }
            if self.rules[..i]
                .iter()
                .any(|r| r.id == rule.id || r.executable == rule.executable)
            {
                return Err(Error::InvalidConfiguration("Règle en double".into()));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ProcessKey {
    pub pid: u32,
    pub creation_time: u64,
    pub session_id: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessInstance {
    pub key: ProcessKey,
    pub executable: ExecutablePath,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PowerSource {
    Ac,
    Battery,
    #[default]
    Unknown,
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub now_ms: u64,
    pub verified: Vec<ProcessInstance>,
    pub uncertain_pids: Vec<u32>,
    pub complete: bool,
    pub power_source: PowerSource,
    pub session_active: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Owner {
    pub user: String,
    pub session_id: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Cycle {
    pub id: u64,
    pub initial: PlanId,
    pub return_plan: PlanId,
    pub last_confirmed: PlanId,
    pub winner: RuleId,
    #[serde(default)]
    pub acquired: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    Prepared,
    Applied,
    Revoked,
    RestorePending,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryRecord {
    pub schema_version: u32,
    pub owner: Owner,
    pub cycle: Cycle,
    pub transaction: u64,
    pub before: PlanId,
    pub intended: PlanId,
    pub phase: Phase,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Power(String),
    Storage(String),
    MissingPlan(PlanId),
    InvalidConfiguration(String),
    ObservationUncertain,
    ForeignOwner,
    StaleTransaction,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Unconfigured,
    Idle,
    Active,
    ExitGrace { deadline_ms: u64 },
    Applying,
    ManualOverride,
    Paused,
    Blocked(Error),
    Recovering,
    SessionInactive,
    Exited,
}
#[derive(Clone, Debug)]
pub struct EngineState {
    pub status: Status,
    pub cycle: Option<Cycle>,
    pub known: BTreeMap<ProcessKey, (ExecutablePath, Option<u64>)>,
    pub revoked_deadline: Option<u64>,
}
impl Default for EngineState {
    fn default() -> Self {
        Self {
            status: Status::Unconfigured,
            cycle: None,
            known: BTreeMap::new(),
            revoked_deadline: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct ViewState {
    pub configuration: Configuration,
    pub plans: Vec<PowerPlan>,
    pub observed_plan: Option<PlanId>,
    pub status: Status,
    pub cycle: Option<Cycle>,
    pub power_source: PowerSource,
    pub open_rules: Vec<RuleId>,
    pub poll_processes: bool,
    pub can_exit: bool,
}
#[derive(Clone, Debug)]
pub enum Command {
    Refresh,
    SaveConfiguration(Configuration),
    Observe(Snapshot),
    Pause,
    Resume,
    Retry,
    Quit,
    QuitAnyway,
}
