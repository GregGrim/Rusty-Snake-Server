use rand::Rng;
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

    pub fn add_player(&mut self, player_id: &str) {
        let new_player = PlayerData::new(player_id);
        self.players.push(new_player); 
    }

    pub fn remove_player(&mut self, player_id: &str) {
        self.players.retain(|player| player.player_id != player_id);
    }

    pub fn remove_lost_players(&mut self) {
        self.players.retain(|player| !player.game_over);
    }

    pub fn set_food(&mut self) {
        self.food = Point::gen_new();
    }
    pub fn get_food(&self) -> &Point {
        &self.food
    }

    pub fn change_player_direction(&mut self, player_id: &str, new_direction: Direction) {
        for player in &mut self.players {
            if player.player_id.as_str() == player_id {
                player.change_direction(new_direction);
                break;
            }
        }
    }

    pub fn move_players(&mut self) {
        for player in &mut self.players {
            player.move_snake();
        }
    }

    pub fn check_players_collision(&mut self) {
        let player_count = self.players.len();

        for i in 0..player_count {
            let head = &self.players[i].snake.body[0].clone();
            
            for j in 0..player_count {
                if i != j {
                    let other_player = &self.players[j];
                    
                    for segment in &other_player.snake.body.clone() {
                        if head.x == segment.x && head.y == segment.y {
                            // Collision detected
                            self.players[i].game_over = true;
                        }
                    }
                }
            }
        }
    }

    pub fn check_player_obstacle_collision(&mut self) {
        for player in &mut self.players {
            player.check_collision();
        }
    }

    pub fn check_players_on_food(&mut self) {
        for player in &mut self.players {
            if player.food_collision(&self.food) {
            self.food = Point::gen_new();
            break;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PlayerData{
    player_id: String,
    snake: Snake,
    score: i32,
    game_over: bool
}

impl PlayerData {
    pub fn new(player_id: &str) -> PlayerData {
        let player = PlayerData {
            player_id: player_id.to_string(),
            snake: Snake::new(Direction::Right, vec![
                Point{x: 3, y: 1},
                Point{x: 2, y: 1},
                Point{x: 1, y: 1}
                ]),
            score: 0,
            game_over: false
        };
        player
    }
    pub fn move_snake(&mut self) {
        self.snake.move_snake()
    }
    pub fn change_direction(&mut self, new_direction: Direction) {
        self.snake.change_direction(new_direction);
    }
    pub fn food_collision(&mut self, food: &Point) -> bool{
        for block in &self.snake.body {
            if block == food {
                self.snake.has_eaten = true;
                self.score += 1;
                return true;
            }
        }
        false
    }
    pub fn check_collision(&mut self) {
        if self.snake.check_collision() {
            self.game_over = true;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Point {
    x: i32,
    y: i32
}

impl Point {
    pub fn gen_new() -> Point{
        let mut rng = rand::thread_rng();
        let x: i32 = rng.gen_range(0..=19);
        let y: i32 = rng.gen_range(0..=19);
        Point {x, y}
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        if self.x == other.x && self.y == other.y {
            true
        } else {
            false
        }
    }
}

impl Eq for Point {}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point {x: self.x ,y: self.y}
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
} 

impl Direction {
    pub fn to_coordinates(&self) -> (i32, i32) {
        match *self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
    pub fn random() -> Direction{
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen_range(1..=4);
        match num {
            1 => Direction::Up,
            2 => Direction::Down,
            3 => Direction::Right,
            4 => Direction::Left,
            _ => unreachable!(),
        }
    }
    pub fn map(s: &str) -> Direction{
        match s {
            "Up" => Direction::Up,
            "Down" => Direction::Down,
            "Left" => Direction::Left,
            "Right" => Direction::Right,
            _ => unreachable!()
        }
    }
    pub fn is_opposite(&self, other: &Direction) -> bool {
        match (self, other) {
            (Direction::Up, Direction::Down) | (Direction::Down, Direction::Up) |
            (Direction::Left, Direction::Right) | (Direction::Right, Direction::Left) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Snake {
    direction: Direction,
    body: Vec<Point>,
    has_eaten: bool  
}
impl Snake {

    pub fn new(
        direction: Direction,
        body: Vec<Point>
    ) -> Snake {
       Snake {direction, body, has_eaten: false} 
    }

    pub fn move_snake(&mut self) {

        if self.has_eaten {
            let new_segment = self.body.last().unwrap();
            self.body.push(new_segment.clone());
            self.has_eaten = false;
        }
        
        for i in (1..self.body.len()).rev() {

            self.body[i].x = self.body[i-1].x;
            self.body[i].y = self.body[i-1].y;
        }  
        self.body[0].x += self.direction.to_coordinates().0;
        self.body[0].y += self.direction.to_coordinates().1;
    }

    pub fn change_direction(&mut self, new_direction: Direction) {
        if !self.direction.is_opposite(&new_direction) {
            self.direction = new_direction
        }
    }

    fn self_collision(& self) -> bool {
        let head = &self.body[0];
        for i in 1..self.body.len() {
            if self.body[i].x == head.x && self.body[i].y == head.y {
                return true;
            }
        }
        false
    }
    fn wall_collision(& self) -> bool {
        let head = &self.body[0];
        head.x < 0 || 
        head.y < 0 || 
        head.x > 19 || 
        head.y > 19
    }

    pub fn check_collision(& self) -> bool {
        self.self_collision() || self.wall_collision()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerAction {
    PlayerConnected,
    PlayerStartedGame(String),
    PlayerChangedDirection(String, Direction)
}