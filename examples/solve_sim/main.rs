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

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let mut app = App::try_from_files(cli.sim, cli.settings)?;

    for i in 0..cli.iterations_max {
        if let Some(_) = app.run()? {
            println!("Found solution in {i} generation");
            return Ok(());
        }
        app.next_population()?;
    }
    Err(format!(
        "Maximal iterations count reached ({})",
        cli.iterations_max
    ))
}
