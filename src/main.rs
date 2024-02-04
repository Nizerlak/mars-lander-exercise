use simulation::init;
use simulation::FlightState;
use simulation::LanderHistory;
use simulation::Thrust;
use std::env;

fn main() -> Result<(), String> {
    let sim_file_path = env::args().nth(1).ok_or("Lacking simulation path argument")?;
    let settings_file_path = env::args().nth(2).ok_or("Lacking settings path argument")?;

    let mut runner = init::json::from_file(sim_file_path, settings_file_path)?;
    let mut flight_histories: Vec<_> = runner
        .current_landers_states()
        .map(|s| LanderHistory::with_initial_state(s.clone()))
        .collect();

    let now = std::time::Instant::now();
    let result = loop {
        match runner
            .iterate(vec![Thrust::new(-55., 2); runner.num_of_landers()])
            .map_err(|e| e.to_string())
        {
            Ok(simulation::ExecutionStatus::InProgress) => flight_histories
                .iter_mut()
                .zip(runner.current_landers_states())
                .zip(runner.current_flight_states())
                .filter_map(|(x, s)| {
                    if let FlightState::Flying = s {
                        Some(x)
                    } else {
                        None
                    }
                })
                .for_each(|(h, s)| h.append_lander_state(s)),
            e => break e,
        }
    };
    let elapsed = now.elapsed();
    println!("Run ended with result: {result:?} time: {elapsed:?}");

    let flight_state = runner.current_flight_states().next().unwrap();
    println!("{}", flight_histories[0].pretty_to_string());

    println!("Finished {:?}", flight_state);
    Ok(())
}
