
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::models;


#[tokio::main]
pub async fn run() {
    let addr = "127.0.0.1:3000".to_string();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Server is up on: {}", addr);

    let (tx, _) = broadcast::channel(10);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.expect("Error during the websocket handshake occurred");

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            loop {
                tokio::select! {
                    Some(msg) = ws_receiver.next() => {
                        let msg = msg.expect("Failed to read message");
                        if msg.is_text() {
                            let msg_text = msg.to_text().unwrap().to_string();
                            if let Ok(game_data) = serde_json::from_str::<models::GameData>(&msg_text) {
                                tx.send((msg_text.clone(), addr)).unwrap();
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

