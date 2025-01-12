use std::path::PathBuf;

use clap::Parser;
use simulation::{App, Chromosome, FlightState, Landing};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Solver settings json file path
    #[arg(long, value_name = "FILE")]
    settings: PathBuf,

    /// Simulation json file path
    #[arg(long, value_name = "FILE")]
    sim: PathBuf,

    /// Maximal number of iterations
    #[arg(long, short, default_value = "1000")]
    iterations_max: usize,
}

fn find_solved(app: &App) -> Option<Chromosome> {
    app.get_current_states()
        .zip(app.get_population_accumulated())
        .find_map(|(state, chromosome)| {
            if let FlightState::Landed(Landing::Correct) = state {
                Some(chromosome)
            } else {
                None
            }
        })
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let mut app = App::try_from_files(cli.sim, cli.settings)?;

    for i in 0..cli.iterations_max {
        app.run()?;
        if let Some(_) = find_solved(&app) {
            println!("Found solution in {i} generation");
            return Ok(());
        }
        app.next_population()?;
    }
    Err("Maximal iterations count reached".to_owned())
}
