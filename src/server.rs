
use std::{collections::HashMap, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::{broadcast, Mutex}, time};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::models::{GameData, PlayerAction};
use serde_json::Result;


#[tokio::main]
pub async fn run() {
    let addr = "127.0.0.1:3000".to_string();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Server is up on: {}", addr);

    let (tx, _) = broadcast::channel(10);

    let game_data = Arc::new(Mutex::new(GameData::new()));
    let active_clients = Arc::new(Mutex::new(HashMap::new()));

    

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let game_data = Arc::clone(&game_data);
        let active_clients = Arc::clone(&active_clients);

        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.expect("Error during the websocket handshake occurred");

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let mut interval = time::interval(Duration::from_millis(200));

            let mut client_id: Option<String> = None;

            loop {
                tokio::select! {
                    Some(msg) = ws_receiver.next() => {
                        match msg {
                            Ok(msg) => {
                                if msg.is_text() {
                                    let msg_text = msg.to_text().unwrap().to_string();
                                    if let Ok(player_action) = serde_json::from_str::<PlayerAction>(&msg_text) {
                                        match player_action {
                                            PlayerAction::PlayerConnected => {
                                                let player_id = uuid::Uuid::new_v4().to_string();
                                                {
                                                    let mut active_clients = active_clients.lock().await;
                                                    active_clients.insert(addr, player_id.clone());
                                                }
                                                client_id = Some(player_id.clone());
                                                let serialized_data = serde_json::to_string(&*player_id).unwrap();
                                                tx.send((serialized_data, addr)).unwrap();
                                            }
                                            PlayerAction::PlayerStartedGame(player_id) => {
                                                let mut game_data = game_data.lock().await;
                                                game_data.add_player(&player_id);
                                                let serialized_data = serde_json::to_string(&*game_data).unwrap();
                                                tx.send((serialized_data, addr)).unwrap();
                                            }
                                            PlayerAction::PlayerChangedDirection(player_id, direction) => {
                                                let mut game_data = game_data.lock().await;
                                                game_data.change_player_direction(&player_id, direction);
                                                let serialized_data = serde_json::to_string(&*game_data).unwrap();
                                                tx.send((serialized_data, addr)).unwrap();
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        // game engine logic
                        let mut game_data = game_data.lock().await;
                        game_data.remove_lost_players();
                        game_data.move_players();
                        game_data.check_players_collision();
                        game_data.check_player_obstacle_collision();
                        game_data.check_players_on_food();
                        let serialized_data = serde_json::to_string(&*game_data).unwrap();
                        tx.send((serialized_data, addr)).unwrap();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        // catch player_id sending
                        let string_result: Result<String> = serde_json::from_str(&msg);
                        if let Ok(_) = string_result {
                            if addr == other_addr {
                                ws_sender.send(Message::text(msg)).await.unwrap();
                            }    
                        } else {
                            // broadcast gamedata to all players
                            ws_sender.send(Message::text(msg)).await.unwrap();
                        }
                    }
                }
            }
            // remove player from gamedata on disconnect
            if let Some(id) = client_id {
                let mut game_data = game_data.lock().await;
                game_data.remove_player(&id);
                let serialized_data = serde_json::to_string(&*game_data).unwrap();
                tx.send((serialized_data, addr)).unwrap();

                let mut active_clients = active_clients.lock().await;
                active_clients.remove(&addr);
            }
            println!("Client disconnected: {}", addr);
        });
    }
}

