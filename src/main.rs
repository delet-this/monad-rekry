use dotenv::dotenv;
use json_types::{GameInstance, GameState, RunCommandData, SubGameData};
use serde_json::Value;
use std::thread;
use std::time::Duration;
use std::{env, net::TcpStream};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

mod json_types;

const FRONTEND_BASE: &str = "noflight.monad.fi";
const BACKEND_BASE: &str = "noflight.monad.fi/backend";

// Normalize any angle to 0-359
const fn normalize_heading(heading: i32) -> i32 {
    (heading + 360) % 360
}

fn get_env_var(name: &str) -> String {
    env::vars()
        .find_map(|(k, v)| if k == name { Some(v) } else { None })
        .unwrap_or_else(|| panic!("Env var '{name}' doesn't exist"))
}

const fn first_steps(_game_state: &GameState) -> Vec<String> {
    vec![]
}

fn turning(game_state: &GameState) -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    let aircraft = &game_state.aircrafts[0];
    let airport = &game_state.airports[0];

    // initial turn towards airport
    if aircraft.direction > 50 {
        let new_dir = normalize_heading(std::cmp::max(aircraft.direction - 20, 45));
        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }
    // correct landing direction at the end
    else if (aircraft.position.y >= airport.position.y - 8.0)
        && aircraft.direction != airport.direction
    {
        let new_dir = normalize_heading(std::cmp::max(aircraft.direction - 20, airport.direction));
        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }

    commands
}

fn loop_around(game_state: &GameState) -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    let aircraft = &game_state.aircrafts[0];
    let airport = &game_state.airports[0];

    // initial turn
    if aircraft.position.x >= 40.0 && aircraft.direction < 198 {
        let new_dir = normalize_heading(std::cmp::min(aircraft.direction + 20, 198));
        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }
    // straighten
    else if aircraft.position.x <= 20.0 && aircraft.position.x >= 8.0 && aircraft.direction != 180
    {
        let new_dir = normalize_heading(180);
        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }
    // correct landing direction at the end
    else if aircraft.position.x <= airport.position.x + 5.0
        && aircraft.direction != airport.direction
    {
        let new_dir = normalize_heading(std::cmp::max(aircraft.direction - 20, airport.direction));
        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }

    commands
}

fn multiplane(game_state: &GameState) -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    let airport = &game_state.airports[0];

    for aircraft in &game_state.aircrafts {
        // initial turn
        if aircraft.position.x >= -50.0 && aircraft.direction < 20 {
            let new_dir = normalize_heading(std::cmp::min(aircraft.direction + 20, 20));
            commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
        }
        // last turn
        else if aircraft.position.x >= airport.position.x - 15.0
            && aircraft.direction != airport.direction
        {
            let new_dir =
                normalize_heading(std::cmp::min(aircraft.direction + 20, airport.direction));
            commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
        }
    }

    commands
}

fn criss_cross(game_state: &GameState) -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    if game_state.aircrafts.len() < 2 {
        return vec![];
    }

    let airport2 = &game_state.airports[1];
    let aircraft1 = &game_state.aircrafts[0];
    let aircraft2 = &game_state.aircrafts[1];

    // initial turn
    if aircraft1.position.x <= -110.0 && aircraft1.direction >= 315 {
        let new_dir = normalize_heading(aircraft1.direction - 10);
        commands.push(format!("HEAD {} {}", aircraft1.id, new_dir));
    }
    // middle turn
    else if aircraft1.position.y < aircraft2.position.y
        && aircraft1.position.x < 50.0
        && aircraft1.direction < 325
    {
        let new_dir = normalize_heading(aircraft1.direction + 20);
        commands.push(format!("HEAD {} {}", aircraft1.id, new_dir));
    }
    // last turn
    else if aircraft1.position.x >= 90.0 && aircraft1.direction != airport2.direction {
        let new_dir =
            normalize_heading(std::cmp::max(aircraft1.direction - 20, airport2.direction));
        commands.push(format!("HEAD {} {}", aircraft1.id, new_dir));
    }

    commands
}

fn wrong_way(game_state: &GameState) -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    let airport = &game_state.airports[0];
    let aircraft = &game_state.aircrafts[0];

    // initial turn
    if aircraft.position.y < 10.0 && aircraft.position.x < 5.0 {
        let new_dir = normalize_heading(std::cmp::min(aircraft.direction + 20, 85));
        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }
    // second turn
    else if (aircraft.position.y >= 29.0 || aircraft.direction > 180)
        && aircraft.direction != airport.direction
    {
        let new_dir = if (airport.direction - aircraft.direction).abs() <= 20 {
            airport.direction
        } else {
            normalize_heading(aircraft.direction - 20)
        };

        commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
    }

    commands
}

fn dont_crash(game_state: &GameState) -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    // let airport = &game_state.airports[0];
    // let aircraft1 = &game_state.aircrafts[0];
    // let aircraft2 = &game_state.aircrafts[1];
    let aircraft3 = &game_state.aircrafts.last().unwrap();

    {
        let init_target_dir = 270 + 20 + 20;
        let turn_back_target_dir = 270 - 20 - 20 - 10;
        let airport_dir = game_state.airports[0].direction;

        // 3 initial turn
        if aircraft3.position.y >= 75.0 && aircraft3.direction != init_target_dir {
            let new_dir =
                normalize_heading(std::cmp::min(aircraft3.direction + 20, init_target_dir));
            commands.push(format!("HEAD {} {}", aircraft3.id, new_dir));
        }
        // 3 turn back
        else if (-11.0..=55.0).contains(&aircraft3.position.y)
            && aircraft3.direction != turn_back_target_dir
        {
            let new_dir = normalize_heading(std::cmp::max(
                aircraft3.direction - 20,
                turn_back_target_dir,
            ));
            commands.push(format!("HEAD {} {}", aircraft3.id, new_dir));
        }
        // 3 end turn
        else if aircraft3.position.y <= -25.0
            && aircraft3.direction != game_state.airports[0].direction
        {
            let new_dir = normalize_heading(std::cmp::min(aircraft3.direction + 20, airport_dir));
            commands.push(format!("HEAD {} {}", aircraft3.id, new_dir));
        }
    }

    if game_state.aircrafts.len() < 3 {
        return commands;
    }

    // 1 and 2 turns
    for i in 0..=1 {
        let aircraft = &game_state.aircrafts[i];
        let airport = &game_state.airports[i + 1];

        // 1 and 2 initial turn
        let init_target_dir = 280;
        if aircraft.position.y >= 75.0 && aircraft.direction != init_target_dir {
            let new_dir =
                normalize_heading(std::cmp::min(aircraft.direction + 20, init_target_dir));
            commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
        }
        // 1 and 2 end turn
        else if (airport.position.y - aircraft.position.y).abs() < 10.0
            && airport.direction != aircraft.direction
        {
            let new_dir =
                normalize_heading(std::cmp::max(aircraft.direction - 20, airport.direction));
            commands.push(format!("HEAD {} {}", aircraft.id, new_dir));
        }
    }

    commands
}

// fn plane_holdem(game_state: &GameState) -> Vec<String> {
//     let mut commands: Vec<String> = Vec::new();
//
//     commands
// }

fn generate_commands(game_state: &GameState) -> Vec<String> {
    if game_state.airports.is_empty() || game_state.aircrafts.is_empty() {
        return vec![];
    }

    match get_env_var("LEVEL_ID").as_str() {
        "01GH1E14E88DT3BYWHNYW85ZRV" => first_steps(game_state),
        "01GH1E14EDA3AK5MQRNP811WME" => turning(game_state),
        "01GH1E14EDD4FRTAGA4P0S80SB" => loop_around(game_state),
        "01GH1E14EEPNDCXVZXAZ6ZKJEC" => multiplane(game_state),
        "01GH1E14EFSJTVQXP0P49FSZA7" => criss_cross(game_state),
        "01GH1E14EFSVXHWHRRSEAJRZ6R" => wrong_way(game_state),
        "01GH1E14EGT9EGBZ1MEWWB4S7M" => dont_crash(game_state),
        // "01GPZZRWH74XDWK98SNJFZV5FV" => plane_holdem(game_state),
        &_ => {
            vec![]
        }
    }
}

fn handle_game_instance(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, data: Value) {
    // Got a game tick, lets parse it
    let game_state_data: GameInstance = serde_json::from_value(data).unwrap();
    let game_state: GameState = serde_json::from_str(&game_state_data.game_state).unwrap();

    // and send back our commands based on game state
    let commands = generate_commands(&game_state);

    thread::sleep(Duration::from_millis(100));
    let response = serde_json::to_string(&(
        "run-command",
        RunCommandData {
            game_id: game_state_data.entity_id,
            payload: commands,
        },
    ))
    .unwrap();
    socket.write_message(Message::text(response)).unwrap();
}

fn handle_socket(mut socket: WebSocket<MaybeTlsStream<TcpStream>>) {
    loop {
        match socket.read_message().unwrap() {
            Message::Text(content) => {
                let message: (&str, Value) = serde_json::from_str(&content).unwrap();
                match message {
                    ("game-instance", data) => handle_game_instance(&mut socket, data),
                    ("success", data) => println!("success: {data}"),
                    ("failure", data) => println!("failure: {data}"),
                    _ => println!("Unhandled message: {message:?}"),
                }
            }
            Message::Close(_) => {
                println!("CLOSED");
                break;
            }
            Message::Ping(data) => socket.write_message(Message::Pong(data)).unwrap(),
            msg => println!("Unhandled message type: {:?}", msg),
        }
    }
}

fn create_game(level_id: &str, token: &str) -> GameInstance {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("https://{BACKEND_BASE}/api/levels/{level_id}"))
        .header(reqwest::header::AUTHORIZATION, token)
        .send()
        .expect("Failed to make post request to sever");

    assert!(
        res.status().is_success(),
        "Couldn't create game: {} - {}",
        res.status().canonical_reason().unwrap(),
        res.text().unwrap()
    );

    serde_json::from_str(&res.text().unwrap()).unwrap()
}

fn main() {
    dotenv().expect("A valid .env file doesn't exist");

    let token = get_env_var("TOKEN");
    let level_id = get_env_var("LEVEL_ID");

    let game_instance = create_game(&level_id, &token);
    let game_id = game_instance.entity_id;

    let game_url = format!("https://{FRONTEND_BASE}/?id={game_id}");
    println!("Game at {game_url}");
    open::that(game_url).unwrap();
    thread::sleep(Duration::from_secs(2));

    let ws_url = format!("wss://{BACKEND_BASE}/{token}/");
    let (mut socket, _) = connect(ws_url).expect("Failed to connect to websocket");
    let sub_message = serde_json::to_string(&("sub-game", SubGameData { id: game_id })).unwrap();
    socket.write_message(Message::text(sub_message)).unwrap();

    handle_socket(socket);
}
