use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde::Serialize;
use serde_json::Value;
use simulation::{App, LanderState};
use std::env;
use tower_http::cors::CorsLayer;

// https://docs.rs/axum/latest/axum/index.html#using-the-state-extractor

#[derive(Serialize, Default)]
struct Telemetry {
    vx: Vec<f64>,
    vy: Vec<f64>,
    fuel: Vec<i32>,
    angle: Vec<f64>,
    power: Vec<i32>,
}

#[derive(Serialize)]
pub enum FlightState {
    Flying,
    LandedCorrectly,
    CrashedWrongTerrain,
    CrashedNotVertical,
    CrashedTooFastVertical,
    CrashedTooFastHorizontal,
    OutOfMap,
}

impl From<&simulation::FlightState> for FlightState {
    fn from(value: &simulation::FlightState) -> Self {
        type FS = simulation::FlightState;
        type L = simulation::Landing;
        match value {
            FS::Flying => Self::Flying,
            FS::Landed(landing) => match landing {
                L::Correct => Self::LandedCorrectly,
                L::WrongTerrain => Self::CrashedWrongTerrain,
                L::NotVertical { error: _ } => Self::CrashedNotVertical,
                L::TooFastVertical { error: _ } => Self::CrashedTooFastVertical,
                L::TooFastHorizontal { error: _ } => Self::CrashedTooFastHorizontal,
                L::OutOfMap => Self::OutOfMap,
            },
        }
    }
}

#[derive(Serialize)]
struct Route {
    telemetry: Telemetry,
    positions: Vec<(f64, f64)>,
    flight_state: FlightState,
}

#[derive(Serialize)]
struct Population {
    id: usize,
    routes: Vec<Route>,
    fitness: Vec<f64>,
}

#[derive(Clone)]
struct AppState {
    state: Arc<Mutex<App>>,
}

#[tokio::main]
async fn main() {
    let sim_file_path = env::args()
        .nth(1)
        .expect("Lacking simulation path argument");
    let settings_file_path = env::args().nth(2).expect("Lacking settings path argument");

    let app = match App::try_new(&sim_file_path, &settings_file_path) {
        Ok(app) => AppState {
            state: Arc::new(Mutex::new(app)),
        },
        Err(e) => panic!("{e}"),
    };
    app.state.lock().unwrap().run().unwrap();

    // build our application with a single route
    let router = Router::new()
        .route("/terrain", get(handle_terrain))
        .route("/population", get(handle_population))
        .route("/next", put(handle_next))
        .route(
            "/reset",
            put(|State(state): State<AppState>| async move {
                let mut app = state.state.lock().unwrap();
                *app = App::try_new(&sim_file_path.clone(), &settings_file_path.clone()).unwrap();
                app.run().unwrap();
            }),
        )
        .with_state(app)
        .layer(CorsLayer::permissive());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn handle_terrain(State(state): State<AppState>) -> Json<Value> {
    let app = state.state.lock().unwrap();
    let terrain = app.get_terrain();
    let v = terrain
        .x
        .iter()
        .zip(terrain.y.iter())
        .map(|(x, y)| vec![x, y])
        .collect::<Vec<_>>();
    Json(serde_json::to_value(v).unwrap())
}

async fn handle_next(State(state): State<AppState>) -> Result<(), (StatusCode, String)> {
    let mut app = state.state.lock().unwrap();
    app.next_population().map_err(|e: String| {
        let e = format!("App next population failed: {e}");
        eprintln!("{e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;
    app.run().map_err(|e: String| {
        let e = format!("App run failed: {e}");
        eprintln!("{e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;
    Ok(())
}

async fn handle_population(State(AppState { state }): State<AppState>) -> Json<Value> {
    let app = state.lock().unwrap();
    let routes = app
        .get_routes()
        .zip(app.get_current_states())
        .map(lander_states_to_route)
        .collect::<Vec<_>>();
    let population = Population {
        id: app.get_population_id(),
        routes,
        fitness: app.get_current_fitness().collect(),
    };
    Json(serde_json::to_value(population).unwrap())
}

fn lander_states_to_route(
    (states, flight_state): (impl Iterator<Item = LanderState>, &simulation::FlightState),
) -> Route {
    states.fold(
        Route {
            positions: Vec::new(),
            telemetry: Telemetry::default(),
            flight_state: FlightState::from(flight_state),
        },
        |mut route, state| {
            let LanderState {
                x,
                y,
                vx,
                vy,
                fuel,
                angle,
                power,
            } = state;
            route.positions.push((x, y));
            route.telemetry.vx.push(vx);
            route.telemetry.vy.push(vy);
            route.telemetry.fuel.push(fuel);
            route.telemetry.angle.push(angle);
            route.telemetry.power.push(power);
            route
        },
    )
}
