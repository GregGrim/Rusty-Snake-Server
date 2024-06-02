use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameData {
    players: Vec<PlayerData>,
    food: Point,
}
impl GameData {
    pub fn new() -> GameData{
        let game_data =  GameData {
            players: vec![],
            food: Point{x:10, y:10}
        };
        game_data
    }

    pub fn update(&mut self, player_data: PlayerData) {
        for player in &mut self.players {
            if player_data.player_id == player.player_id {
                *player = player_data;
                return;
            }
        }
        self.players.push(player_data);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerData{
    player_id: String,
    snake_position: Vec<Point>,
    score: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Point{
    x: i32,
    y: i32
}
