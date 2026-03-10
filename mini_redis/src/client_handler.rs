use serde_json::{self, json};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};

use crate::command::{decr, del, expire, get, incr, keys, ping, save, set, ttl, Request, Response};
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
                    Err(_) => {
                        let resp = Response {
                            status: "error".to_string(),
                            message: Some("invalid json".to_string()),
                            ..Default::default()
                        };
                        if let Err(e) = write_half
                            .write_all((serde_json::to_string(&resp).unwrap()).as_bytes())
                            .await
                        {
                            eprintln!("Error writing to socket: {}", e);
                            break;
                        }
                        continue;
                    }
                };

                let result = match req.cmd.as_str() {
                    "PING" => ping().await,
                    "SET" => {
                        set(
                            req.key.unwrap_or_default().to_string(),
                            req.value.unwrap_or_default(),
                            store.clone(),
                        )
                        .await
                    }
                    "GET" => get(req.key.unwrap_or_default().to_string(), store.clone()).await,
                    "DEL" => del(req.key.unwrap_or_default().to_string(), store.clone()).await,
                    "KEYS" => keys(store.clone()).await,
                    "EXPIRE" => expire(store.clone()).await,
                    "TTL" => ttl(store.clone()).await,
                    "INCR" => incr(req.key.unwrap_or_default().to_string(), store.clone()).await,
                    "DECR" => decr(req.key.unwrap_or_default().to_string(), store.clone()).await,
                    "SAVE" => save(store.clone()).await,
                    _ => Ok(Response {
                        status: "error".to_string(),
                        message: Some("unknown command".to_string()),
                        ..Default::default()
                    }),
                };

                let resp = match result {
                    Ok(r) => r,
                    Err(_) => Response {
                        status: "error".to_string(),
                        message: Some("unknown error".to_string()),
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
