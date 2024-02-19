use crate::{init, simulation::*};
use rand::Rng;

struct DummyCommandProvider {}

impl CommandProvider for DummyCommandProvider {
    fn get_cmd(&self, _: usize, _: usize) -> Option<Thrust> {
        Some(Thrust::new(-55., 2 + rand::thread_rng().gen_range(0..=1)))
    }
}

pub struct App {
    terrain: Terrain,
    lander_runner: LanderRunner,
    initial_lander_state: LanderState,
    flight_histories: Vec<LanderHistory>,
    cmd_provider: DummyCommandProvider,
}

impl App {
    pub fn try_new(sim_file_path: String, settings_file_path: String) -> Result<Self, String> {
        let (initial_lander_state, terrain) = init::json::parse_sim(sim_file_path)?;
        let settings = init::json::parse_settings(settings_file_path)?;
        let lander_runner = LanderRunner::new(
            initial_lander_state.clone(),
            settings.num_of_runners,
            Physics::default(),
            CollisionChecker::default(),
        );
        let flight_histories: Vec<_> =
            vec![
                LanderHistory::with_initial_state(initial_lander_state.clone());
                settings.num_of_runners
            ];
        let cmd_provider = DummyCommandProvider {};

        Ok(Self {
            terrain,
            lander_runner,
            initial_lander_state,
            flight_histories,
            cmd_provider,
        })
    }

    pub fn reset(&mut self) {
        self.lander_runner
            .reinitialize(self.initial_lander_state.clone());
        self.flight_histories.iter_mut().for_each(|h| {
            *h = LanderHistory::with_initial_state(self.initial_lander_state.clone())
        });
    }

    pub fn run(&mut self) -> Result<ExecutionStatus, String> {
        let res = loop {
            match self
                .lander_runner
                .iterate(&self.cmd_provider, &self.terrain)
                .map_err(|e| e.to_string())
            {
                Ok(ExecutionStatus::InProgress) => self.save_last_lander_states_in_flight(),
                e => break e,
            }
        };
        self.save_last_lander_states();
        res
    }

    pub fn get_routes(&self) -> impl Iterator<Item = impl Iterator<Item = LanderState> + '_> + '_ {
        self.flight_histories.iter().map(|h| h.iter_history())
    }

    pub fn get_current_states(&self) -> impl Iterator<Item = &FlightState> + '_ {
        self.lander_runner
            .current_flight_states()
    }

    pub fn get_terrain(&self) -> &Terrain {
        &self.terrain
    }

    fn save_last_lander_states_in_flight(&mut self) {
        self.flight_histories
            .iter_mut()
            .zip(self.lander_runner.current_landers_states())
            .zip(self.lander_runner.current_flight_states())
            .filter_map(|(x, s)| {
                if let FlightState::Flying = s {
                    Some(x)
                } else {
                    None
                }
            })
            .for_each(|(h, s)| h.append_lander_state(s))
    }

    fn save_last_lander_states(&mut self) {
        self.flight_histories
            .iter_mut()
            .zip(self.lander_runner.current_landers_states())
            .for_each(|(h, s)| h.append_lander_state(s))
    }

    pub fn print_flight_state_results(&self) {
        println!("{}", self.flight_histories[0].pretty_to_string());

        let flight_state = self.lander_runner.current_flight_states().next().unwrap();
        println!("Finished {:?}", flight_state);
    }
}
