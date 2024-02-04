use std::{fs::File, io::Read};

use json::{self, JsonValue};

use crate::{CollisionChecker, LanderRunner, LanderState, Physics, Terrain};

macro_rules! get_json {
    ($json:ident,$($key:literal),+, $func:ident) => {
            get_json!($json$([$key])+, concat!($("/",$key),+), $func)
    };

    ($value:expr, $key:expr, $func:ident) => {
        $value
            .$func()
            .ok_or(concat!("Couldn't find ", $key))?
    };
}

macro_rules! json_value_or_err {
    ($json:ident,$($key:literal),+) => {
            {
                let value = &$json$([$key])+;
                if value.is_null() {
                    Err(concat!("Lacking", concat!($("/",$key),+), " key"))
                }else{
                    Ok(value)
                }
            }
    };
}

pub fn from_file(file_path: String) -> Result<LanderRunner, String> {
    let json = read_json(file_path)?;
    Ok(LanderRunner::new(
        parse_lander(&json)?,
        get_json!(json, "NumOfRunners", as_usize),
        Physics::default(),
        CollisionChecker::default(),
        parse_terrain(json_value_or_err!(json, "Terrain")?)?,
    ))
}

fn read_json(file_path: String) -> Result<JsonValue, String> {
    let mut file_content = String::new();
    let mut file = File::open(file_path).map_err(|e| format!("Error while opening file: {e}"))?;

    file.read_to_string(&mut file_content)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    json::parse(&file_content).map_err(|e| format!("Json error: {e}"))
}

fn parse_terrain(terrain_array: &JsonValue) -> Result<Terrain, String> {
    terrain_array
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
        .map(|(x, y)| Terrain { x, y })
}

fn parse_lander(json: &JsonValue) -> Result<LanderState, String> {
    let mut initial_lander_state = LanderState::default();

    initial_lander_state.x = get_json!(json, "Lander", "X", as_f64);
    initial_lander_state.y = get_json!(json, "Lander", "Y", as_f64);
    initial_lander_state.vx = get_json!(json, "Lander", "HSpeed", as_f64);
    initial_lander_state.vy = get_json!(json, "Lander", "VSpeed", as_f64);
    initial_lander_state.fuel = get_json!(json, "Lander", "Fuel", as_i32);
    initial_lander_state.angle = get_json!(json, "Lander", "Angle", as_f64);
    initial_lander_state.power = get_json!(json, "Lander", "Power", as_i32);
    Ok(initial_lander_state)
}
