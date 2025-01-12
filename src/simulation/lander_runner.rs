use std::fmt::Display;

pub use crate::simulation::*;

#[derive(Debug)]
pub enum Error {
    InconsistentState,
    CommandGetError { id: usize, sub_id: usize },
    SimulationError(SimulationError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum ExecutionStatus {
    InProgress,
    Finished,
}

#[derive(Debug, Clone)]
pub enum FlightState {
    Flying,
    Landed(Landing),
}

impl From<SimulationError> for Error {
    fn from(val: SimulationError) -> Self {
        Error::SimulationError(val)
    }
}

struct LanderStateCalculation {
    physics: Physics,
    collision_checker: CollisionChecker,
}

impl LanderStateCalculation {
    pub fn new(physics: Physics, collision_checker: CollisionChecker) -> Self {
        Self {
            physics,
            collision_checker,
        }
    }

    pub fn calculate_new_lander_state(
        &self,
        lander: &LanderState,
        cmd: Command,
    ) -> Result<(LanderState, FlightState), Error> {
        let new_lander_state = self
            .physics
            .iterate(lander.clone(), cmd)
            .map_err(<SimulationError as std::convert::Into<Error>>::into)?;
        if let Some(((x, y), landing)) = self.collision_checker.check(lander, &new_lander_state) {
            Ok((
                LanderState {
                    x,
                    y,
                    ..new_lander_state
                },
                FlightState::Landed(landing),
            ))
        } else {
            Ok((new_lander_state, FlightState::Flying))
        }
    }
}

pub struct LanderRunner {
    states: Vec<FlightState>,
    landers: Vec<LanderState>,
    lander_state_calculator: LanderStateCalculation,
    angle_step: f64,
    iteration_id: usize,
}

impl LanderRunner {
    pub fn new(
        initial_lander_state: LanderState,
        num_of_landers: usize,
        physics: Physics,
        collision_checker: CollisionChecker,
    ) -> Self {
        let angle_step = collision_checker.angle_step;
        Self {
            lander_state_calculator: LanderStateCalculation::new(physics, collision_checker),
            states: vec![FlightState::Flying; num_of_landers],
            landers: vec![initial_lander_state; num_of_landers],
            angle_step,
            iteration_id: 0,
        }
    }

    pub fn reinitialize(&mut self, initial_lander_state: LanderState) {
        self.states = vec![FlightState::Flying; self.num_of_landers()];
        self.landers = vec![initial_lander_state; self.num_of_landers()];
        self.iteration_id = 0;
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

    pub fn iterate(&mut self, population: &mut [Chromosome]) -> Result<ExecutionStatus, Error> {
        assert_eq!(self.states.len(), self.landers.len());
        assert_eq!(self.states.len(), population.len());

        let mut picked_any = false;

        for (id, ((lander, flight_state), angle_thrust)) in self
            .landers
            .iter_mut()
            .zip(self.states.iter_mut())
            .zip(
                population
                    .iter_mut()
                    .map(|chromosome| get_id_or_last(chromosome, self.iteration_id)),
            )
            .enumerate()
        {
            if let FlightState::Flying = *flight_state {
                picked_any = true;
                let (angle, thrust) = angle_thrust.ok_or(Error::CommandGetError {
                    id,
                    sub_id: self.iteration_id,
                })?;
                let (new_lander_state, new_flight_state) = match self
                    .lander_state_calculator
                    .calculate_new_lander_state(lander, Command::new(*angle as f64, *thrust))?
                {
                    (_, FlightState::Landed(Landing::NotVertical { error_abs, .. }))
                        if error_abs <= self.angle_step =>
                    {
                        *angle = 0;
                        println!("Corrected angle for chromosome with id {id}");
                        self.lander_state_calculator.calculate_new_lander_state(
                            lander,
                            Command::new(*angle as f64, *thrust),
                        )? // recalculate for new angle
                    }
                    other => other,
                };
                *lander = new_lander_state;
                *flight_state = new_flight_state;
            }
        }

        if picked_any {
            self.iteration_id += 1;
            Ok(ExecutionStatus::InProgress)
        } else {
            Ok(ExecutionStatus::Finished)
        }
    }
}
fn get_id_or_last(chromosome: &mut Chromosome, index: usize) -> Option<(&mut i32, &mut i32)> {
    if index < chromosome.angles.len() {
        Some((
            chromosome.angles.get_mut(index)?,
            chromosome.thrusts.get_mut(index)?,
        )) // Safe because we checked bounds
    } else {
        Some((
            chromosome.angles.last_mut()?,
            chromosome.thrusts.last_mut()?,
        )) // Fallback to last_mut
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

    pub fn iter_history(&self) -> impl Iterator<Item = LanderState> + '_ {
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
