use std::env;
use simulation::init;
use simulation::Thrust;


fn main() -> Result<(), String> {
    let file_path = env::args().nth(1).ok_or("Lacking path argument")?;

    let mut runner = init::json::from_file(file_path)?;

    let result = loop {
        match runner
            .iterate(vec![Thrust::new(-55.,2);runner.num_of_landers()])
            .map_err(|e| e.to_string())
        {
            Ok(simulation::ExecutionStatus::InProgress) => (),
            e => break e,
        }
    };
    println!("Run ended with result: {result:?}");

    let (flight_state, flight_history) = runner.current_states().next().unwrap();
    println!("{}", flight_history.pretty_to_string());

    println!("Finished {:?}", flight_state);
    Ok(())
}


