use simulation::{App, LanderState, Settings, Terrain};

fn simple_app() -> App {
    App::try_new(
        LanderState::default()
            .with_y(1000.)
            .with_x(500.)
            .with_fuel(1000),
        Terrain::with_default_limits(vec![0., 1000.], vec![0., 0.]),
        Settings {
            population_size: 300,
            chromosome_size: 50,
            elitism: 0.2,
            mutation_prob: 0.01,
        },
    )
    .unwrap()
}

#[test]
fn simple_run() {
    let mut app = simple_app();

    for i in 0..10 {
        let print_err = |e: &String| println!("Failed on {i} iteration: {e}");
        app.run().inspect_err(print_err).unwrap();
        app.next_population().inspect_err(print_err).unwrap();
    }
}
