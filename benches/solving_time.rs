use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simulation::App;
use std::env;
use std::path::PathBuf;

// Unfortunately hackery needed to take input args via env vars

fn get_env_var_or_default(var_name: &str, default: &str) -> String {
    env::var(var_name)
        .unwrap_or_else(|_| default.to_string())
        .to_string()
}

fn get_env_path_or_default(var_name: &str, default: &str) -> PathBuf {
    let path_str = get_env_var_or_default(var_name, default);
    PathBuf::from(path_str)
}

fn solve(mut app: App, max_iterations: usize) {
    for _ in 0..max_iterations {
        if let Some(_) = app.run().unwrap() {
            return;
        }
        app.next_population().unwrap();
    }
    panic!("Max iterations ({max_iterations}) reached");
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let settings = get_env_path_or_default("SOLVER_SETTINGS", "default_settings.json");
    let sim = get_env_path_or_default("SIMULATION_FILE", "default_simulation.json");
    let iterations_max: usize = get_env_var_or_default("MAX_ITERATIONS", "1000")
        .parse()
        .unwrap_or(1000);

    c.bench_function("solving_time", |b| {
        b.iter(|| {
            solve(
                black_box(App::try_from_files(sim.clone(), settings.clone()).unwrap()),
                black_box(iterations_max),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
