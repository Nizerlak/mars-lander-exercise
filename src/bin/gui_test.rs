use std::sync::{Arc, Mutex};

use axum::{extract::State, response::Json, routing::get, Router};
use serde::Serialize;
use serde_json::Value;
use simulation::{App, LanderState};
use std::env;
use tower_http::cors::CorsLayer;

// https://docs.rs/axum/latest/axum/index.html#using-the-state-extractor

#[derive(Serialize)]
struct Telemetry {
    vx: f64,
    vy: f64,
    fuel: i32,
    angle: f64,
    power: i32,
}

#[derive(Serialize)]
struct Route {
    telemetry: Vec<Telemetry>,
    positions: Vec<(f64, f64)>,
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

    let app = match App::try_new(sim_file_path, settings_file_path) {
        Ok(app) => AppState {
            state: Arc::new(Mutex::new(app)),
        },
        Err(e) => panic!("{e}"),
    };

    // build our application with a single route
    let router = Router::new()
        .route("/terrain", get(handle_terrain))
        .route("/routes", get(handle_routes))
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

async fn handle_routes(State(state): State<AppState>) -> Json<Value> {
    let mut app = state.state.lock().unwrap();
    let _ = app.run();
    let routes = app
        .get_routes()
        .map(lander_states_to_route)
        .collect::<Vec<_>>();
    Json(serde_json::to_value(routes).unwrap())
}

fn lander_states_to_route(states: impl Iterator<Item = LanderState>) -> Route {
    states.fold(
        Route {
            positions: Vec::new(),
            telemetry: Vec::new(),
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
            route.telemetry.push(Telemetry {
                vx,
                vy,
                fuel,
                angle,
                power,
            });
            route
        },
    )
}
