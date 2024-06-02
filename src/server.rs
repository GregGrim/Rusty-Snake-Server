
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::{broadcast, Mutex}};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::models::{self, GameData};


#[tokio::main]
pub async fn run() {
    let addr = "127.0.0.1:3000".to_string();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Server is up on: {}", addr);

    let (tx, _) = broadcast::channel(10);

    let game_data = Arc::new(Mutex::new(GameData::new()));

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let game_data = Arc::clone(&game_data);

        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.expect("Error during the websocket handshake occurred");

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            loop {
                tokio::select! {
                    Some(msg) = ws_receiver.next() => {
                        let msg = msg.expect("Failed to read message");
                        if msg.is_text() {
                            let msg_text = msg.to_text().unwrap().to_string();
                            if let Ok(player_data) = serde_json::from_str::<models::PlayerData>(&msg_text) {
                                let mut game_data = game_data.lock().await;
                                game_data.update(player_data);
                                let serialized_data = serde_json::to_string(&*game_data).unwrap();
                                tx.send((serialized_data, addr)).unwrap();
                            }
                        }
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            ws_sender.send(Message::text(msg)).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}

