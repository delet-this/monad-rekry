use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubGameData {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Airport {
    pub name: String,
    pub position: Point,
    pub direction: i32,
    pub landing_radius: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Aircraft {
    pub id: String,
    pub name: String,
    pub position: Point,
    pub direction: i32,
    pub speed: f64,
    pub collision_radius: f64,
    pub destination: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub bbox: [Point; 2],
    pub airports: Vec<Airport>,
    pub aircrafts: Vec<Aircraft>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameInstance {
    pub game_state: String,
    pub status: String,
    pub reason: String,
    pub created_at: String,
    pub game_type: String,
    pub entity_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunCommandData {
    pub game_id: String,
    pub payload: Vec<String>,
}
