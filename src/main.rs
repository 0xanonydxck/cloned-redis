mod command;
mod state;
mod storage;
mod utils;

use state::{State, DEFAULT_ADDR, FLAG_ADDR, FLAG_DBFILENAME, FLAG_DIR};
use storage::CacheData;

use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

#[tokio::main]
async fn main() {
    let args = utils::parse_flag(vec![
        FLAG_DIR.to_string(),
        FLAG_DBFILENAME.to_string(),
        FLAG_ADDR.to_string(),
    ]);

    let listener = TcpListener::bind(args.get(FLAG_ADDR).unwrap_or(&DEFAULT_ADDR.to_string()))
        .await
        .unwrap();
    let storage: Arc<RwLock<HashMap<String, CacheData>>> = Arc::new(RwLock::new(HashMap::new()));
    let state: Arc<RwLock<State>> = Arc::new(RwLock::new(State::new(
        args.get(FLAG_DIR).cloned(),
        args.get(FLAG_DBFILENAME).cloned(),
    )));

    println!("redis server started..");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let storage_cloned = Arc::clone(&storage);
        let state_cloned = Arc::clone(&state);

        tokio::spawn(async move {
            command::process_stream(stream, state_cloned, storage_cloned).await;
        });
    }
}
