use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameData {
    player_id: u32,
    position: (i32, i32),
    direction: String,
    score: u32,
}