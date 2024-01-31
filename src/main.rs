use std::{env, fs::File, io::Read};

use json::{self, JsonValue};
use simulation::{CollisionChecker, LanderState, Physics, Runner, Terrain, Thrust};

fn main() -> Result<(), String> {
    let file_path = env::args().nth(1).ok_or("Lacking path argument")?;

    let json = read_json(file_path)?;

    let mut runner = runner_from_json(json)?;

    loop {
        if !runner
            .iterate(vec![Thrust::new(-55.,2)])
            .map_err(|e| e.to_string())?
        {
            break;
        }
    }
    println!("{}", runner.landers[0].pretty_to_string());

    println!("Finished {:?}", runner.states[0]);
    Ok(())
}

fn read_json(file_path: String) -> Result<JsonValue, String> {
    let mut file_content = String::new();
    let mut file = File::open(file_path).map_err(|_| "Path does not exist")?;

    file.read_to_string(&mut file_content)
        .map_err(|_| "Failed to read file")?;
    json::parse(&file_content).map_err(|e| e.to_string())
}

fn runner_from_json(json: JsonValue) -> Result<Runner, String> {
    let mut initial_lander_state = LanderState::default();

    macro_rules! get_json {
        ($member:ident,$key:literal, $func:ident) => {
            initial_lander_state.$member = json["Lander"][$key]
                .$func()
                .ok_or("Couldn't find Lander/".to_owned() + $key)?;
        };
    }

    get_json!(x, "X", as_f64);
    get_json!(y, "Y", as_f64);
    get_json!(vx, "HSpeed", as_f64);
    get_json!(vy, "VSpeed", as_f64);
    get_json!(fuel, "Fuel", as_i32);
    get_json!(angle, "Angle", as_f64);
    get_json!(power, "Power", as_i32);

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

    Ok(Runner::new(
        initial_lander_state,
        1,
        Physics::default(),
        CollisionChecker::default(),
        terrain,
    ))
}
