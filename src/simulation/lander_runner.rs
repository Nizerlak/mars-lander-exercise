pub use crate::physics::SimulationError;
use crate::{
    physics::{LanderState, Physics, Thrust},
    CollisionChecker, Landing, Terrain,
};

#[derive(Debug)]
pub enum Error {
    InconsistentState,
    WrongNumberOfCommands { expected: usize, got: usize },
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
    landers: Vec<LanderHistory>,
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
        let lander_history = LanderHistory::with_initial_state(initial_lander_state);
        Self {
            physics,
            collision_checker,
            terrain,
            states: vec![FlightState::Flying; num_of_landers],
            landers: vec![lander_history; num_of_landers],
            executions_left: None,
        }
    }

    pub fn executions_limit(self, limit: usize) -> Self {
        Self {
            executions_left: Some(limit),
            ..self
        }
    }

    pub fn num_of_landers(&self) -> usize{
        self.landers.len()
    }

    pub fn current_states(&self) -> impl Iterator<Item = (&FlightState, &LanderHistory)> {
        self.states.iter().zip(&self.landers)
    }

    pub fn iterate(&mut self, cmds: Vec<Thrust>) -> Result<ExecutionStatus, Error> {
        assert_eq!(self.states.len(), self.landers.len());

        if cmds.len() != self.landers.len() {
            return Err(Error::WrongNumberOfCommands {
                expected: self.landers.len(),
                got: cmds.len(),
            });
        }

        if let Some(0) = self.executions_left {
            return Ok(ExecutionStatus::ExecutionLimitReached);
        }

        let mut picked_any = false;

        for ((lander_history, cmd), flight_state) in self
            .landers
            .iter_mut()
            .zip(cmds)
            .zip(self.states.iter_mut())
        {
            if let FlightState::Flying = *flight_state {
                picked_any = true;
                let lander = lander_history
                    .last_lander_state()
                    .ok_or(Error::InconsistentState)?;
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
                lander_history.append_lander_state(new_lander_state);
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
    fn with_initial_state(state: LanderState) -> Self {
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

    fn last_lander_state(&self) -> Option<LanderState> {
        let x = *self.x.last()?;
        let y = *self.y.last()?;
        let vx = *self.vx.last()?;
        let vy = *self.vy.last()?;
        let fuel = *self.fuel.last()?;
        let angle = *self.angle.last()?;
        let power = *self.power.last()?;
        Some(LanderState {
            x,
            y,
            vx,
            vy,
            fuel,
            angle,
            power,
        })
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

    fn append_lander_state(&mut self, state: LanderState) {
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
