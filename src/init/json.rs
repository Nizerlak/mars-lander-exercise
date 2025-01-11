use crate::simulation::*;
use json::{self, JsonValue};
use std::{fs::File, io::Read};

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

pub fn parse_settings(settings_file_path: &String) -> Result<Settings, String> {
    let settings_json = read_json(settings_file_path)?;

    let settings = Settings {
        population_size: get_json!(settings_json, "PopulationSize", as_usize),
        chromosome_size: get_json!(settings_json, "ChromosomeSize", as_usize),
        elitism: get_json!(settings_json, "Elitism", as_f64),
        mutation_prob: get_json!(settings_json, "MutationProb", as_f64),
    };
    Ok(settings)
}

pub fn parse_sim(sim_file_path: &String) -> Result<(LanderState, Terrain), String> {
    let sim_json = read_json(sim_file_path)?;

    Ok((
        parse_lander(&sim_json)?,
        parse_terrain(json_value_or_err!(sim_json, "Terrain")?)?,
    ))
}

fn read_json(file_path: &String) -> Result<JsonValue, String> {
    let mut file_content = String::new();
    let mut file =
        File::open(file_path).map_err(|e| format!("Error while opening file {file_path}: {e}"))?;

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
        .map(|(x, y)| Terrain::with_default_limits(x, y))
}

fn parse_lander(json: &JsonValue) -> Result<LanderState, String> {
    Ok(LanderState {
        x: get_json!(json, "Lander", "X", as_f64),
        y: get_json!(json, "Lander", "Y", as_f64),
        vx: get_json!(json, "Lander", "HSpeed", as_f64),
        vy: get_json!(json, "Lander", "VSpeed", as_f64),
        fuel: get_json!(json, "Lander", "Fuel", as_i32),
        angle: get_json!(json, "Lander", "Angle", as_f64),
        power: get_json!(json, "Lander", "Power", as_i32),
    })
}
