pub use crate::physics::SimulationError;
use crate::{
    physics::{LanderState, Physics},
    solver::CommandProvider,
    CollisionChecker, Landing, Terrain,
};

#[derive(Debug)]
pub enum Error {
    InconsistentState,
    CommandGetError { id: usize },
    SimulationError(SimulationError),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug)]
pub enum ExecutionStatus {
    InProgress,
    Finished,
    ExecutionLimitReached,
}

#[derive(Debug, Clone)]
pub enum FlightState {
    Flying,
    Landed(Landing),
    OutsideMap,
}

impl Into<Error> for SimulationError {
    fn into(self) -> Error {
        Error::SimulationError(self)
    }
}

pub struct LanderRunner {
    states: Vec<FlightState>,
    landers: Vec<LanderState>,
    physics: Physics,
    collision_checker: CollisionChecker,
    terrain: Terrain,
    executions_left: Option<usize>,
}

impl LanderRunner {
    pub fn new(
        initial_lander_state: LanderState,
        num_of_landers: usize,
        physics: Physics,
        collision_checker: CollisionChecker,
        terrain: Terrain,
    ) -> Self {
        Self {
            physics,
            collision_checker,
            terrain,
            states: vec![FlightState::Flying; num_of_landers],
            landers: vec![initial_lander_state; num_of_landers],
            executions_left: None,
        }
    }

    pub fn executions_limit(self, limit: usize) -> Self {
        Self {
            executions_left: Some(limit),
            ..self
        }
    }

    pub fn num_of_landers(&self) -> usize {
        self.landers.len()
    }

    pub fn current_landers_states(&self) -> impl Iterator<Item = &LanderState> {
        self.landers.iter()
    }

    pub fn current_flight_states(&self) -> impl Iterator<Item = &FlightState> {
        self.states.iter()
    }

    pub fn iterate(
        &mut self,
        command_provider: &impl CommandProvider,
    ) -> Result<ExecutionStatus, Error> {
        assert_eq!(self.states.len(), self.landers.len());

        if let Some(0) = self.executions_left {
            return Ok(ExecutionStatus::ExecutionLimitReached);
        }

        let mut picked_any = false;

        for (id, (lander, flight_state)) in self
            .landers
            .iter_mut()
            .zip(self.states.iter_mut())
            .enumerate()
        {
            if let FlightState::Flying = *flight_state {
                picked_any = true;
                let cmd = command_provider
                    .get_cmd(id)
                    .ok_or(Error::CommandGetError { id })?;
                let new_lander_state = self
                    .physics
                    .iterate(lander.clone(), cmd)
                    .map_err(|e| e.into())?;
                if let Some(landing) =
                    self.collision_checker
                        .check(&self.terrain, &lander, &new_lander_state)
                {
                    *flight_state = FlightState::Landed(landing);
                }
                *lander = new_lander_state;
            }
        }

        if picked_any {
            if let Some(ref mut exectuions_left) = self.executions_left {
                *exectuions_left -= 1;
            }
            Ok(ExecutionStatus::InProgress)
        } else {
            Ok(ExecutionStatus::Finished)
        }
    }
}

#[derive(Clone)]
pub struct LanderHistory {
    x: Vec<f64>,
    y: Vec<f64>,
    vx: Vec<f64>,
    vy: Vec<f64>,
    fuel: Vec<i32>,
    angle: Vec<f64>,
    power: Vec<i32>,
}

impl LanderHistory {
    pub fn with_initial_state(state: LanderState) -> Self {
        let LanderState {
            x,
            y,
            vx,
            vy,
            fuel,
            angle,
            power,
        } = state;
        Self {
            x: vec![x],
            y: vec![y],
            vx: vec![vx],
            vy: vec![vy],
            fuel: vec![fuel],
            angle: vec![angle],
            power: vec![power],
        }
    }

    pub fn pretty_to_string(&self) -> String {
        self.iter_history().fold(
            format!(
                "{:8}{:8}{:8}{:8}{:8}{:8}{:8}",
                "X", "Y", "VX", "VY", "FUEL", "ANGLE", "POWER"
            ),
            |out,
             LanderState {
                 x,
                 y,
                 vx,
                 vy,
                 fuel,
                 angle,
                 power,
             }| {
                out + &format!("\n{x:5.2} {y:5.2} {vx:5.2} {vy:5.2} {fuel:7} {angle:7} {power:7}")
            },
        )
    }

    pub fn append_lander_state(&mut self, state: &LanderState) {
        self.x.push(state.x);
        self.y.push(state.y);
        self.vx.push(state.vx);
        self.vy.push(state.vy);
        self.fuel.push(state.fuel);
        self.angle.push(state.angle);
        self.power.push(state.power);
    }

    fn iter_history(&self) -> impl Iterator<Item = LanderState> + '_ {
        self.x
            .iter()
            .zip(&self.y)
            .zip(&self.vx)
            .zip(&self.vy)
            .zip(&self.fuel)
            .zip(&self.angle)
            .zip(&self.power)
            .map(|((((((x, y), vx), vy), fuel), angle), power)| LanderState {
                x: *x,
                y: *y,
                vx: *vx,
                vy: *vy,
                fuel: *fuel,
                angle: *angle,
                power: *power,
            })
    }
}
