
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex, io::{AsyncBufReadExt, BufReader, AsyncWriteExt}};
use serde_json::{self, json};

use crate::command::{Request, Response, del, get, ping, set};
type Store = Arc<Mutex<HashMap<String, String>>>;

pub async fn handle_client(socket: TcpStream, store: Store) {
    let (read_half, mut write_half) = socket.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {

            Ok(0) => {
                break;
            }

            Ok(_) => {
                let req: Request = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(e) => {
                        let resp = Response {
                            status: "error".to_string(),
                            message: Some(format!("Invalid JSON: {}", e)),
                            ..Default::default()
                        };
                        if let Err(e) = write_half.write_all((serde_json::to_string(&resp).unwrap()).as_bytes()).await {
                            eprintln!("Error writing to socket: {}", e);
                            break;
                        }
                        continue;
                    }
                };

                let result = match req.cmd.as_str() {
                    "PING" => ping().await,
                    "SET" => set(req.key.unwrap_or_default().to_string(), req.value.unwrap_or_default(), store.clone()).await,
                    "GET" => get(req.key.unwrap_or_default().to_string(), store.clone()).await,
                    "DEL" => del(req.key.unwrap_or_default().to_string(), store.clone()).await,
                    _ => Ok(Response {
                        status: "error".to_string(),
                        message: Some("Unknown command".to_string()),
                        ..Default::default()
                    }),
                };

                let resp = match result {
                    Ok(r) => r,
                    Err(_) => Response {
                        status: "error".to_string(),
                        message: Some("Unknown error".to_string()),
                        ..Default::default()
                    },
                };

                let json = serde_json::to_string(&resp).unwrap() + "\n";

                if let Err(e) = write_half.write_all(json.as_bytes()).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}
