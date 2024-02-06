use rand::Rng;
use simulation::init;
use simulation::CommandProvider;
use simulation::ExecutionStatus;
use simulation::FlightState;
use simulation::LanderHistory;
use simulation::LanderRunner;
use simulation::Thrust;
use std::env;

struct DummyCommandProvider {}

impl CommandProvider for DummyCommandProvider {
    fn get_cmd(&self, _: usize) -> Option<Thrust> {
        Some(Thrust::new(-55., 2 + rand::thread_rng().gen_range(0..=1)))
    }
}

struct App {
    lander_runner: LanderRunner,
    flight_histories: Vec<LanderHistory>,
    cmd_provider: DummyCommandProvider,
}

impl App {
    fn try_new(sim_file_path: String, settings_file_path: String) -> Result<Self, String> {
        let lander_runner = init::json::from_file(sim_file_path, settings_file_path)?;
        let flight_histories: Vec<_> = lander_runner
            .current_landers_states()
            .map(|s| LanderHistory::with_initial_state(s.clone()))
            .collect();
        let cmd_provider = DummyCommandProvider {};

        Ok(Self {
            lander_runner,
            flight_histories,
            cmd_provider,
        })
    }

    fn run(&mut self) -> Result<ExecutionStatus, String> {
        loop {
            match self
                .lander_runner
                .iterate(&self.cmd_provider)
                .map_err(|e| e.to_string())
            {
                Ok(simulation::ExecutionStatus::InProgress) => self.save_last_lander_states(),
                e => break e,
            }
        }
    }

    fn save_last_lander_states(&mut self) {
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

    fn print_flight_state_results(&self) {
        println!("{}", self.flight_histories[0].pretty_to_string());

        let flight_state = self.lander_runner.current_flight_states().next().unwrap();
        println!("Finished {:?}", flight_state);
    }
}

fn main() -> Result<(), String> {
    let sim_file_path = env::args()
        .nth(1)
        .ok_or("Lacking simulation path argument")?;
    let settings_file_path = env::args().nth(2).ok_or("Lacking settings path argument")?;

    let mut app = App::try_new(sim_file_path, settings_file_path)?;
    let now = std::time::Instant::now();
    let result = app.run();
    let elapsed = now.elapsed();
    println!("Run ended with result: {result:?} time: {elapsed:?}");

    app.print_flight_state_results();
    Ok(())
}
