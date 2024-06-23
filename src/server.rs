use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::{broadcast, Mutex}, time};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::{models::{Direction, GameData}, utils::current_timestamp};

pub fn game_step(
    game_data: &mut GameData
) {
    game_data.remove_lost_players();
    game_data.move_players();
    game_data.check_players_collision();
    game_data.check_player_obstacle_collision();
    game_data.check_players_on_food();
}

pub async fn player_connected_action(
    active_clients: &Arc<Mutex<HashMap<SocketAddr, String>>>,
    addr: SocketAddr
) -> String {
    let player_id = uuid::Uuid::new_v4().to_string();
    {
        let mut active_clients = active_clients.lock().await;
        active_clients.insert(addr, player_id.clone());
    }
    player_id
}

#[tokio::main]
pub async fn run() {
    let addr = "0.0.0.0:3000".to_string();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Server is up on: {};  {}", addr, current_timestamp());

    let (tx, _) = broadcast::channel(10);

    let game_data = Arc::new(Mutex::new(GameData::new()));
    let active_clients = Arc::new(Mutex::new(HashMap::new()));


    tokio::spawn({
        let game_data = Arc::clone(&game_data);
        let tx = tx.clone();
        async move {
            let mut interval = time::interval(Duration::from_millis(200));
            loop {
                interval.tick().await;
                let mut game_data = game_data.lock().await;
                game_step(&mut game_data);
                let serialized_data = serde_json::to_string(&*game_data).unwrap();
                if tx.send((serialized_data, "game_data".to_string())).is_err() {
                    // eprintln!("Error: Failed to broadcast game data");
                }
            }
        }
    });

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection: {};  {}", addr, current_timestamp());

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let game_data = Arc::clone(&game_data);
        let active_clients = Arc::clone(&active_clients);

        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.expect("Error during the websocket handshake occurred");

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let mut client_id: Option<String> = None;

            loop {
                tokio::select! {
                    Some(msg) = ws_receiver.next() => {
                        let msg = msg.unwrap().to_text().unwrap().to_string();
                        if let Ok(request) = serde_json::from_str::<HashMap<String, String>>(&msg) {
                            if let Some(action) = request.get("action") {
                                match action.as_str() {
                                    "player_connected" => {
                                        let player_id = player_connected_action(&active_clients, addr).await;

                                        client_id = Some(player_id.clone());
                                        let serialized_data = serde_json::to_string(&*player_id).unwrap();
                                        if tx.send((serialized_data, addr.to_string())).is_err() {
                                            eprintln!("Error: Failed to send player_id");
                                        }
                                    }
                                    "player_started_game" => {
                                        let mut game_data = game_data.lock().await;
                                        game_data.add_player(request.get("player_id").unwrap());

                                        let serialized_data = serde_json::to_string(&*game_data).unwrap();
                                        if tx.send((serialized_data, addr.to_string())).is_err() {
                                            eprintln!("Error: Failed to send game data");
                                        }
                                    }
                                    "player_changed_direction" => {
                                        let mut game_data = game_data.lock().await;
                                        game_data.change_player_direction(
                                            request.get("player_id").unwrap(),
                                            Direction::map(request.get("direction").unwrap())
                                        );
                                        let serialized_data = serde_json::to_string(&*game_data).unwrap();
                                        if tx.send((serialized_data, addr.to_string())).is_err() {
                                            eprintln!("Error: Failed to send game data");
                                        }
                                    }
                                    _ => unreachable!()
                                }
                            }
                        }
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if other_addr == "game_data" {
                            match ws_sender.send(Message::text(msg)).await {
                                Ok(_) => {},
                                Err(_) => {
                                    break;
                                }
                            };
                        } else if addr.to_string() == other_addr {
                            ws_sender.send(Message::text(msg)).await.expect(
                                format!("Failed to send player_id to {};  {}", addr, current_timestamp()).as_str()
                            );
                        }
                    }
                }
            }

            // remove player from gamedata on disconnect
            if let Some(id) = client_id {
                let mut game_data = game_data.lock().await;
                game_data.remove_player(&id);
                let serialized_data = serde_json::to_string(&*game_data).unwrap();
                if tx.send((serialized_data, "game_data".to_string())).is_err() {
                   // eprintln!("Error: Failed to broadcast game data");
                }

                let mut active_clients = active_clients.lock().await;
                active_clients.remove(&addr);
            }
            println!("Player disconnected: {};  {}", addr, current_timestamp());
        });
    }
}
