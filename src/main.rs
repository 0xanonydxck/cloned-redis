mod command;
mod service;
mod state;
mod storage;

use std::{collections::HashMap, env, sync::Arc};

use state::State;
use storage::CacheData;
use tokio::{net::TcpListener, sync::RwLock};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Initialize variables to hold values of --dir and --dbfilename
    let mut dir = None;
    let mut dbfilename = None;

    // Iterate over the arguments to find --dir and --dbfilename
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                if i + 1 < args.len() {
                    dir = Some(args[i + 1].clone());
                }
                i += 1; // Skip the next argument since it's the value
            }
            "--dbfilename" => {
                if i + 1 < args.len() {
                    dbfilename = Some(args[i + 1].clone());
                }
                i += 1; // Skip the next argument since it's the value
            }
            _ => {}
        }
        i += 1;
    }

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let state: Arc<RwLock<State>> = Arc::new(RwLock::new(State::new(dir, dbfilename)));
    let storage: Arc<RwLock<HashMap<String, CacheData>>> = Arc::new(RwLock::new(HashMap::new()));
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
