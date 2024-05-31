use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    Chat {
        sender: String,
        recipient: String,
        content: String,
        timestamp: String,
    },
    UserConnected {
        sender: String,
        content: String,
        timestamp: String,
        client_id: String
    },
    UserDisconnected {
        sender: String,
        content: String,
        timestamp: String,
    },
    System {
        sender: String,
        content: String,
        timestamp: String,
    },
}