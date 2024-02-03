use simulation::init;
use simulation::LanderHistory;
use simulation::Thrust;
use std::env;

fn main() -> Result<(), String> {
    let file_path = env::args().nth(1).ok_or("Lacking path argument")?;

    let mut runner = init::json::from_file(file_path)?;
    let mut flight_histories: Vec<_> = runner
        .current_states()
        .map(|(_, s)| LanderHistory::with_initial_state(s.clone()))
        .collect();

    let now = std::time::Instant::now();
    let result = loop {
        match runner
            .iterate(vec![Thrust::new(-55., 2); runner.num_of_landers()])
            .map_err(|e| e.to_string())
        {
            Ok(simulation::ExecutionStatus::InProgress) => flight_histories
                .iter_mut()
                .zip(runner.current_states())
                .for_each(|(h, (_, s))| h.append_lander_state(s)),
            e => break e,
        }
    };
    let elapsed = now.elapsed();
    println!("Run ended with result: {result:?} time: {elapsed:?}");

    let (flight_state, _) = runner.current_states().next().unwrap();
    println!("{}", flight_histories[0].pretty_to_string());

    println!("Finished {:?}", flight_state);
    Ok(())
}
