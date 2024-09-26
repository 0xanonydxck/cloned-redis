use std::{collections::HashMap, sync::Arc};

use cmd::Command;
use router::{
    config_router, echo_router, get_router, ping_router, set_router, CONFIG_ROUTER, ECHO_ROUTER,
    GET_ROUTER, PING_ROUTER, SET_ROUTER,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::RwLock,
};

use crate::{state::State, storage::CacheData};

pub mod cmd;
pub mod router;

const MESSAGE_SIZE: usize = 1024;

pub async fn process_stream(
    mut stream: TcpStream,
    state: Arc<RwLock<State>>,
    storage: Arc<RwLock<HashMap<String, CacheData>>>,
) {
    let mut buf = [0; MESSAGE_SIZE];

    loop {
        let message_size = stream.read(&mut buf).await.unwrap();
        if message_size == 0 {
            break;
        }

        let message = String::from_utf8(buf.to_vec()).unwrap();
        command_routing(&mut stream, state.clone(), storage.clone(), message)
            .await
            .unwrap();

        buf = [0; MESSAGE_SIZE];
    }
}

async fn command_routing(
    stream: &mut TcpStream,
    state: Arc<RwLock<State>>,
    storage: Arc<RwLock<HashMap<String, CacheData>>>,
    message: String,
) -> Result<(), ()> {
    let cmd = match Command::build(message) {
        Ok(cmd) => cmd,
        Err(e) => {
            // TODO - error response
            eprintln!("{:?}", e);
            stream.write(b"+invalid input\r\n").await.unwrap();
            return Ok(());
        }
    };

    match cmd.command.as_str() {
        PING_ROUTER => ping_router(stream).await.unwrap(),
        ECHO_ROUTER => echo_router(stream, cmd).await.unwrap(),
        GET_ROUTER => get_router(stream, storage, cmd).await.unwrap(),
        SET_ROUTER => set_router(stream, storage, cmd).await.unwrap(),
        CONFIG_ROUTER => config_router(stream, state, cmd).await.unwrap(),
        _ => {
            stream.write(b"+command not found\r\n").await.unwrap();
        }
    }

    Ok(())
}
