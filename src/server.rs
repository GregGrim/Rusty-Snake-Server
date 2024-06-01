use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpListener, sync::broadcast};

use crate::{message::Message, utils::current_timestamp};



#[tokio::main]
pub async fn run(){
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("server is up on: {}", addr);

    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("New Server connection: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 { // TODO change logic 
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap()
                        }
                    }
                }
            }

        });
    }
}

