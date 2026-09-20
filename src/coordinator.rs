use crate::{
    domain::*,
    engine::{self, Intent},
    storage::Store,
};

pub trait PowerPort {
    fn active_plan(&mut self) -> Result<PlanId, Error>;
    fn set_active_plan(&mut self, plan: PlanId) -> Result<(), Error>;
    fn plans(&mut self) -> Result<Vec<PowerPlan>, Error>;
}
pub struct Controller<P: PowerPort, S: Store> {
    power: P,
    store: S,
    owner: Owner,
    configuration: Configuration,
    plans: Vec<PowerPlan>,
    state: EngineState,
    observed: Option<PlanId>,
    record: Option<RecoveryRecord>,
    snapshot: Snapshot,
    next_id: u64,
}
impl<P: PowerPort, S: Store> Controller<P, S> {
    pub fn open(mut power: P, mut store: S, owner: Owner) -> Result<Self, Error> {
        let configuration = store.load_configuration()?;
        let record = store.load_recovery()?;
        if record.as_ref().is_some_and(|r| r.owner.user != owner.user) {
            return Err(Error::ForeignOwner);
        }
        let plans = power.plans()?;
        let observed = Some(power.active_plan()?);
        let state = EngineState {
            status: if record.is_some() {
                Status::Recovering
            } else if configuration.paused {
                Status::Paused
            } else {
                Status::Unconfigured
            },
            cycle: record.as_ref().map(|r| r.cycle.clone()),
            ..Default::default()
        };
        let clock_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::Storage(e.to_string()))?
            .as_nanos() as u64;
        let next_id = record.as_ref().map_or(clock_id, |r| {
            clock_id.max(r.cycle.id.max(r.transaction).saturating_add(1))
        });
        Ok(Self {
            power,
            store,
            owner,
            configuration,
            plans,
            state,
            observed,
            record,
            snapshot: Snapshot::default(),
            next_id,
        })
    }
    pub fn view(&self) -> ViewState {
        ViewState {
            configuration: self.configuration.clone(),
            plans: self.plans.clone(),
            observed_plan: self.observed,
            status: self.state.status.clone(),
            cycle: self.state.cycle.clone(),
            poll_processes: self.configuration.automation_enabled
                && !self.configuration.paused
                && self.configuration.rules.iter().any(|r| r.enabled)
                && !matches!(
                    self.state.status,
                    Status::Blocked(_) | Status::SessionInactive | Status::Exited
                ),
            can_exit: self.state.cycle.is_none() || self.state.status == Status::Exited,
        }
    }
    pub fn handle(&mut self, command: Command) -> Result<ViewState, Error> {
        if let Err(error) = self.handle_inner(command) {
            self.state.status = Status::Blocked(error.clone());
            return Err(error);
        }
        Ok(self.view())
    }
    fn handle_inner(&mut self, command: Command) -> Result<(), Error> {
        let explicit = !matches!(command, Command::Observe(_));
        match command {
            Command::Refresh => {
                self.plans = self.power.plans()?;
                let actual = self.read()?;
                if !matches!(
                    self.state.status,
                    Status::Blocked(_)
                        | Status::Recovering
                        | Status::Exited
                        | Status::ManualOverride
                ) && self
                    .state
                    .cycle
                    .as_ref()
                    .is_some_and(|c| c.last_confirmed != actual)
                {
                    self.revoke()?;
                }
                return Ok(());
            }
            Command::QuitAnyway => {
                self.state.status = Status::Exited;
                return Ok(());
            }
            Command::Quit => {
                self.restore(Status::Exited)?;
                return Ok(());
            }
            Command::SaveConfiguration(mut c) => {
                if self.configuration.automation_enabled && !c.rules.iter().any(|r| r.enabled) {
                    c.automation_enabled = false;
                }
                c.validate()?;
                self.store.save_configuration(&c)?;
                self.configuration = c;
            }
            Command::Pause => {
                let mut c = self.configuration.clone();
                c.paused = true;
                self.store.save_configuration(&c)?;
                self.configuration = c;
                self.restore(Status::Paused)?;
                return Ok(());
            }
            Command::Resume => {
                let mut c = self.configuration.clone();
                c.paused = false;
                self.store.save_configuration(&c)?;
                self.configuration = c;
                if self
                    .record
                    .as_ref()
                    .is_some_and(|r| r.phase == Phase::Revoked)
                {
                    self.clear()?;
                }
                self.state.status = if self.record.is_some() {
                    Status::Recovering
                } else {
                    Status::Idle
                };
            }
            Command::Retry => {
                self.plans = self.power.plans()?;
                self.state.status = if self.record.is_some() {
                    Status::Recovering
                } else {
                    Status::Idle
                };
            }
            Command::Observe(snapshot) => {
                self.snapshot = snapshot;
                if matches!(self.state.status, Status::Blocked(_) | Status::Exited) {
                    return Ok(());
                }
            }
        }
        if self.state.status == Status::Recovering {
            self.recover()?;
        }
        if self.state.cycle.is_some() && self.state.status != Status::ManualOverride {
            let actual = self.read()?;
            if Some(actual) != self.state.cycle.as_ref().map(|c| c.last_confirmed) {
                self.revoke()?;
            }
        }
        match engine::decide(
            &mut self.state,
            &self.configuration,
            &self.snapshot,
            explicit,
        ) {
            Intent::None => {
                if self.state.cycle.is_none() || self.state.status == Status::ManualOverride {
                    self.read()?;
                }
                Ok(())
            }
            Intent::Apply { rule, plan } => self.apply(rule, plan),
            Intent::Restore(status) => self.restore(status),
            Intent::CloseRevoked => {
                self.clear()?;
                self.state.status = Status::Idle;
                Ok(())
            }
        }
    }
    fn read(&mut self) -> Result<PlanId, Error> {
        let plan = self.power.active_plan()?;
        self.observed = Some(plan);
        Ok(plan)
    }
    fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }
    fn clear(&mut self) -> Result<(), Error> {
        if let Some(r) = &self.record {
            self.store.clear(r.cycle.id, r.transaction)?;
        }
        self.record = None;
        self.state.cycle = None;
        self.state.revoked_deadline = None;
        Ok(())
    }
    fn revoke(&mut self) -> Result<(), Error> {
        if let Some(mut record) = self.record.clone() {
            record.phase = Phase::Revoked;
            self.store.finalize(&record)?;
            self.record = Some(record);
        }
        self.state.status = Status::ManualOverride;
        Ok(())
    }
    fn apply(&mut self, rule: RuleId, plan: PlanId) -> Result<(), Error> {
        let before = self.read()?;
        let cycle = if let Some(c) = &self.state.cycle {
            if c.last_confirmed != before {
                return self.revoke();
            }
            c.clone()
        } else {
            Cycle {
                id: self.id(),
                initial: before,
                return_plan: self
                    .configuration
                    .return_plan
                    .ok_or_else(|| Error::InvalidConfiguration("Plan de retour manquant".into()))?,
                last_confirmed: before,
                winner: rule,
            }
        };
        let mut cycle = cycle;
        cycle.winner = rule;
        self.transaction(cycle, before, plan, false)?;
        if self.state.status != Status::ManualOverride {
            self.state.status = Status::Active;
        }
        Ok(())
    }
    fn restore(&mut self, status: Status) -> Result<(), Error> {
        if self
            .record
            .as_ref()
            .is_some_and(|r| r.phase == Phase::Revoked)
        {
            self.clear()?;
            self.state.status = status;
            return Ok(());
        }
        if let Some(cycle) = self.state.cycle.clone() {
            let before = self.read()?;
            if before != cycle.last_confirmed {
                self.revoke()?;
                self.clear()?;
                self.state.status = status;
                return Ok(());
            }
            let target = cycle.return_plan;
            self.transaction(cycle, before, target, true)?;
            if self.state.status == Status::ManualOverride {
                self.clear()?;
            }
        }
        self.state.status = status;
        Ok(())
    }
    fn transaction(
        &mut self,
        cycle: Cycle,
        before: PlanId,
        intended: PlanId,
        restore: bool,
    ) -> Result<(), Error> {
        self.plans = self.power.plans()?;
        if !self.plans.iter().any(|p| p.id == intended) {
            return Err(Error::MissingPlan(intended));
        }
        let mut r = RecoveryRecord {
            schema_version: 1,
            owner: self.owner.clone(),
            cycle,
            transaction: self.id(),
            before,
            intended,
            phase: if restore {
                Phase::RestorePending
            } else {
                Phase::Prepared
            },
        };
        self.store.prepare(&r)?;
        self.record = Some(r.clone());
        self.state.cycle = Some(r.cycle.clone());
        self.state.status = Status::Applying;
        if self.read()? != before {
            return self.revoke();
        }
        if before != intended {
            self.power.set_active_plan(intended)?;
        }
        if self.read()? != intended {
            return self.revoke();
        }
        r.cycle.last_confirmed = intended;
        r.phase = Phase::Applied;
        self.store.finalize(&r)?;
        self.state.cycle = Some(r.cycle.clone());
        self.record = Some(r);
        if restore {
            self.clear()?;
        }
        Ok(())
    }
    fn recover(&mut self) -> Result<(), Error> {
        let Some(r) = self.record.clone() else {
            self.state.status = Status::Idle;
            return Ok(());
        };
        if r.phase == Phase::Revoked {
            self.state.status = Status::ManualOverride;
            return Ok(());
        }
        let actual = self.read()?;
        match r.phase {
            Phase::Prepared if actual == r.before || actual == r.intended => {
                self.state.cycle.as_mut().unwrap().last_confirmed = actual;
            }
            Phase::Applied if actual == r.cycle.last_confirmed => {}
            Phase::RestorePending if actual == r.intended => {
                self.clear()?;
                self.state.status = Status::Idle;
                return Ok(());
            }
            Phase::RestorePending if actual == r.cycle.last_confirmed => {
                return self.restore(Status::Idle);
            }
            _ => {
                return self.revoke();
            }
        }
        self.state.status = Status::Active;
        let intent = engine::decide(&mut self.state, &self.configuration, &self.snapshot, true);
        match intent {
            Intent::Apply { rule, plan } => self.apply(rule, plan),
            Intent::Restore(status) => self.restore(status),
            _ => Ok(()),
        }
    }
}
