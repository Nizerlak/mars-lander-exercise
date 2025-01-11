use simulation::App;
use std::env;

fn main() -> Result<(), String> {
    let sim_file_path = env::args()
        .nth(1)
        .expect("Lacking simulation path argument");
    let settings_file_path = env::args().nth(2).expect("Lacking settings path argument");

    let mut app = App::try_from_files_(&sim_file_path, &settings_file_path)?;

    let now = std::time::Instant::now();
    let result = app.run();
    let elapsed = now.elapsed();

    println!("Run ended with result: {result:?} time: {elapsed:?}");

    app.print_flight_state_results();
    Ok(())
}
