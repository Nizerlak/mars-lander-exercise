use std::{fs::File, io::Read};

use json::{self, JsonValue};

use crate::{CollisionChecker, LanderState, Physics, LanderRunner, Terrain};

pub fn from_file(file_path: String)-> Result<LanderRunner, String>{
    runner_from_json(read_json(file_path)?)
}

fn read_json(file_path: String) -> Result<JsonValue, String> {
    let mut file_content = String::new();
    let mut file = File::open(file_path).map_err(|_| "Path does not exist")?;

    file.read_to_string(&mut file_content)
        .map_err(|_| "Failed to read file")?;
    json::parse(&file_content).map_err(|e| e.to_string())
}

fn runner_from_json(json: JsonValue) -> Result<LanderRunner, String> {
    let mut initial_lander_state = LanderState::default();

    macro_rules! get_json {
        ($key:literal, $func:ident) => {
            json["Lander"][$key]
                .$func()
                .ok_or("Couldn't find Lander/".to_owned() + $key)?
        };
    }

    initial_lander_state.x = get_json!("X", as_f64);
    initial_lander_state.y = get_json!("Y", as_f64);
    initial_lander_state.vx = get_json!("HSpeed", as_f64);
    initial_lander_state.vy = get_json!("VSpeed", as_f64);
    initial_lander_state.fuel = get_json!("Fuel", as_i32);
    initial_lander_state.angle = get_json!("Angle", as_f64);
    initial_lander_state.power = get_json!("Power", as_i32);

    let terrain_array = &json["Terrain"];
    if terrain_array.is_null() {
        return Err("Lacking Terrain key".to_owned());
    }

    let terrain = terrain_array
        .members()
        .map(|point_json| {
            let err_str = "Terrain has to contain numeric landpoints";
            let x = point_json[0].as_f64().ok_or(err_str)?;
            let y = point_json[1].as_f64().ok_or(err_str)?;
            Ok((x, y))
        })
        .try_fold(
            (Vec::new(), Vec::new()),
            |(mut xs, mut ys), xy: Result<(f64, f64), String>| {
                let (x, y) = xy?;
                xs.push(x);
                ys.push(y);
                Ok::<(Vec<f64>, Vec<f64>), String>((xs, ys))
            },
        )
        .map(|(x, y)| Terrain { x, y })?;

    Ok(LanderRunner::new(
        initial_lander_state,
        1,
        Physics::default(),
        CollisionChecker::default(),
        terrain,
    ))
}