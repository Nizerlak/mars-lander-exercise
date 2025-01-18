use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simulation::{init, App, Settings};

const SIMPLE_SIM: &str = r#"{
    "Lander": {
        "X": 2500,
        "Y": 2700,
        "HSpeed": 0,
        "VSpeed": 0,
        "Fuel": 550,
        "Angle": 0,
        "Power": 0
    },
    "Terrain": [
        [0,100],
        [1000,500],
        [1500,1500],
        [3000,1000],
        [4000,150],
        [5500,150],
        [6999,800]
    ]
}"#;

const COMPLICATED_SIM: &str = r#"{
    "Lander":{
        "X": 6500,
        "Y": 2000,
        "HSpeed": 0,
        "VSpeed": 0,
        "Fuel": 1200,
        "Angle": 0,
        "Power": 0
    },
    "Terrain": [
        [0,1800],
        [300,1200],
        [1000,1550],
        [2000,1200],
        [2500,1650],
        [3700,220],
        [4700,220],
        [4750,1000],
        [4700,1650],
        [4000,1700],
        [3700,1600],
        [3750,1900],
        [4000,2100],
        [4900,2050],
        [5100,1000],
        [5500,500],
        [6200,800],
        [6999,600]
    ]
}
"#;

fn light_settings() -> Settings {
    Settings {
        chromosome_size: 20,
        population_size: 30,
        elitism: 0.15,
        mutation_prob: 0.01,
    }
}

fn hard_settings() -> Settings {
    Settings {
        population_size: 200,
        chromosome_size: 160,
        elitism: 0.15,
        mutation_prob: 0.01,
    }
}

fn run(app: &mut App) {
    app.run().unwrap();
}

fn run_next_population(app: &mut App) {
    app.run().unwrap();
    app.next_population().unwrap();
}

fn simple_sim() -> (simulation::LanderState, simulation::Terrain) {
    init::json::parse_from_string(SIMPLE_SIM).unwrap()
}

fn complicated_sim() -> (simulation::LanderState, simulation::Terrain) {
    init::json::parse_from_string(COMPLICATED_SIM).unwrap()
}

fn to_app(
    (lander_state, terrain): (simulation::LanderState, simulation::Terrain),
    settings: Settings,
) -> Result<App, String> {
    App::try_new(lander_state, terrain, settings)
}

pub fn run_benchmark(c: &mut Criterion) {
    let mut do_bench = |fun: fn(&mut App), sim, setting, name| {
        let mut app = to_app(sim, setting).unwrap();
        c.bench_function(name, |b| b.iter(|| fun(black_box(&mut app))));
    };
    macro_rules! bench {
        ($func:ident, $arg1:ident, $arg2:ident) => {{
            // Generate a descriptive name automatically based on the input
            let name = concat!(
                stringify!($func),
                "_",
                stringify!($arg1),
                "_",
                stringify!($arg2)
            );
            do_bench($func, $arg1(), $arg2(), name);
        }};
    }

    bench!(run, simple_sim, light_settings);
    bench!(run, complicated_sim, light_settings);
    bench!(run, simple_sim, hard_settings);
    bench!(run, complicated_sim, hard_settings);
    bench!(run_next_population, simple_sim, light_settings);
    bench!(run_next_population, complicated_sim, light_settings);
    bench!(run_next_population, simple_sim, hard_settings);
    bench!(run_next_population, complicated_sim, hard_settings);
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
