use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;

type Store = Arc<Mutex<HashMap<String, String>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request{
    pub cmd: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub seconds: Option<i32>
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Response{
    pub status: String,
    pub value: Option<String>,
    pub count: Option<i32>,
    pub keys: Option<Vec<String>>,
    pub ttl: Option<i32>,
    pub message: Option<String>
}

#[derive(Debug)]
pub struct CmdError;

pub fn parse_request(raw: &str) -> Option<Request> {
    match serde_json::from_str(raw) {
        Ok(req) => Some(req),
        Err(_) => None,
    }
}

pub async fn ping() -> Result<Response, CmdError> {
    Ok(Response {
        status: "ok".to_string(),
        ..Default::default()
    })
}

pub async fn set(key: String, value: String, store: Store) -> Result<Response, CmdError> {
    let mut store = store.lock().await;
    store.insert(key, value);
    Ok(Response {
        status: "ok".to_string(),
        ..Default::default()
    })
}

pub async fn get(key: String, store: Store) -> Result<Response, CmdError> {
    let store = store.lock().await;
    let value = store.get(&key).map(|s| s.to_string());
    Ok(Response {
        status: "ok".to_string(),
        value,
        ..Default::default()
    })
}

pub async fn del(key: String, store: Store) -> Result<Response, CmdError> {
    let mut store = store.lock().await;
    let count = if store.remove(&key).is_some() { 1 } else { 0 };
    Ok(Response {
        status: "ok".to_string(),
        count: Some(count),
        ..Default::default()
    })
}