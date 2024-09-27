use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Ok, Result};
use tokio::net::TcpStream;

use super::*;

pub const PING_ROUTER: &str = "PING";
pub const ECHO_ROUTER: &str = "ECHO";
pub const SET_ROUTER: &str = "SET";
pub const GET_ROUTER: &str = "GET";
pub const CONFIG_ROUTER: &str = "CONFIG";

pub async fn ping_router(stream: &mut TcpStream) -> Result<()> {
    stream.write(b"+PONG\r\n").await.unwrap();
    Ok(())
}

pub async fn echo_router(stream: &mut TcpStream, cmd: Command) -> Result<()> {
    match cmd.value {
        Some(value) => {
            let size = value.len();
            let reply = format!("${}\r\n{}\r\n", size, value);
            stream.write(reply.as_bytes()).await.unwrap();
        }
        None => {
            stream.write(b"+EMPTY\r\n").await.unwrap();
        }
    }

    Ok(())
}

pub async fn get_router(
    stream: &mut TcpStream,
    storage: Arc<RwLock<HashMap<String, CacheData>>>,
    cmd: Command,
) -> Result<()> {
    match cmd.value {
        Some(key) => {
            let mut storage_write = storage.write().await;
            let cache_opt = storage_write.get(key.as_str());

            match cache_opt {
                Some(cache) => {
                    if cache.clone().is_expired() {
                        storage_write.remove(key.as_str());
                        stream.write(b"$-1\r\n").await.unwrap();
                        return Ok(());
                    }

                    let reply_msg = String::from_utf8(cache.value.clone()).unwrap();
                    let reply_size = reply_msg.len();
                    let reply = format!("${}\r\n{}\r\n", reply_size, reply_msg);
                    stream.write(reply.as_bytes()).await.unwrap();
                }
                None => {
                    stream.write(b"+not found\r\n").await.unwrap();
                }
            }
        }
        None => {
            stream.write(b"$-1\r\n").await.unwrap();
        }
    }

    Ok(())
}

pub async fn set_router(
    stream: &mut TcpStream,
    storage: Arc<RwLock<HashMap<String, CacheData>>>,
    cmd: Command,
) -> Result<()> {
    let mut storage_write = storage.write().await;
    let mut data = CacheData {
        value: cmd.value.unwrap().as_bytes().to_vec(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        lifetime: None,
    };

    match cmd.option.get("PX") {
        Some(value) => {
            data.lifetime = match value.to_string().parse::<u128>() {
                Result::Ok(value) => Some(value),
                Err(_) => {
                    stream
                        .write(b"+cannot parse cache lifetime into u128\r\n")
                        .await
                        .unwrap();
                    return Ok(());
                }
            };
        }
        None => (),
    }

    storage_write.insert(cmd.key.unwrap(), data);
    stream.write(b"+OK\r\n").await.unwrap();
    Ok(())
}

pub async fn config_router(
    stream: &mut TcpStream,
    state: Arc<RwLock<State>>,
    cmd: Command,
) -> Result<()> {
    let state_read = state.read().await;

    // TODO - routing (GET, SET, ...)
    match cmd.value.unwrap_or("".to_string()).as_str() {
        "dir" => {
            let reply = format!(
                "*2\r\n$3\r\ndir\r\n${}\r\n{}\r\n",
                state_read.directory.len(),
                state_read.directory
            );

            stream.write(reply.as_bytes()).await.unwrap();
        }
        "dbfilename" => {
            let reply = format!(
                "*2\r\n$3\r\ndir\r\n${}\r\n{}\r\n",
                state_read.filename.len(),
                state_read.filename
            );

            stream.write(reply.as_bytes()).await.unwrap();
        }
        _ => {
            stream.write(b"$-1\r\n").await.unwrap();
        }
    }
    Ok(())
}
