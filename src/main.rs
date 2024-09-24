mod command;
mod service;
mod storage;

use std::{collections::HashMap, sync::Arc};

use storage::CacheData;
use tokio::{net::TcpListener, sync::RwLock};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let storage: Arc<RwLock<HashMap<String, CacheData>>> = Arc::new(RwLock::new(HashMap::new()));
    println!("redis server started..");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let storage_cloned = Arc::clone(&storage);
        tokio::spawn(async move {
            command::process_stream(stream, storage_cloned).await;
        });
    }
}
