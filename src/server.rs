use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use serde_json::from_str;
use tokio::{net::TcpListener, sync::{mpsc, RwLock}};
use tokio_tungstenite::{accept_async, tungstenite::{self, Error as Err}};
use tungstenite::Result as Res;
use futures_util::{StreamExt, SinkExt};
use uuid::Uuid;

use crate::{message::Message, utils::current_timestamp};

type Users = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<tungstenite::Message>>>>;

async fn accept_connection(peer: SocketAddr, stream: tokio::net::TcpStream, users: Users) {
    if let Err(e) = handle_connection(peer, stream, users).await {
        match e {
            Err::ConnectionClosed | Err::Protocol(_) | Err::Utf8 => (),
            err => eprintln!("Error processing connection: {:?}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: tokio::net::TcpStream, users: Users) -> Res<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");

    println!("New WebSocket connection: {}", peer);

    let (tx, mut rx) = mpsc::unbounded_channel();

    

    let (mut write, mut read) = ws_stream.split();

    let user_id = Uuid::new_v4().to_string();
    users.write().await.insert(user_id.clone(), tx);

    let user_connected_msg = Message::UserConnected {
        sender: "server".to_string(),
        content: format!("User {} has connected.", user_id),
        timestamp: current_timestamp(),
        client_id: user_id
    };

    broadcast_message(&users, &user_connected_msg).await;

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_text().unwrap();
                    if let Ok(chat_msg) = from_str::<Message>(text) {
                        match chat_msg {
                            Message::Chat { recipient, .. } => {
                                println!("Received message: {}", text);
                                if let Some(tx) = users.read().await.get(&recipient) {
                                    let _ = tx.send(tungstenite::Message::Text(text.to_string()));
                                    println!("Sent message: {}", text);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

async fn broadcast_message(users: &Users, message: &Message) {
    let msg_text = serde_json::to_string(&message).unwrap();
    for user in users.read().await.values() {
        let _ = user.send(tungstenite::Message::Text(msg_text.clone()));
    }
}

#[tokio::main]
pub async fn run(){
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Listening on: {}", addr);

    let users: Users = Arc::new(RwLock::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        let users = users.clone();
        tokio::spawn(accept_connection(peer, stream, users));
    }
}

