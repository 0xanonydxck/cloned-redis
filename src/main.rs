use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(Clone, Debug)]
struct Data {
    value: String,
    created_at: u128,
    expired_at: Option<u128>,
}

/*
    TODO - refactor how to extract the redis-command
    TODO - refactor `message dispatcher`
    TODO - research `can we extract the buffer more than 512 bytes?`
*/
async fn process_stream(mut stream: TcpStream) {
    let mut storage: HashMap<String, Data> = HashMap::new();
    let mut buf: [u8; 512] = [0; 512];

    loop {
        let read_count = stream.read(&mut buf).await.unwrap();
        if read_count == 0 {
            return;
        }

        let lines = String::from_utf8(buf.to_vec()).unwrap();
        let messages: Vec<&str> = lines.split("\r\n").into_iter().collect();
        let command = messages.get(2).unwrap().to_string().to_uppercase();

        match command.as_str() {
            "PING" => {
                stream.write(b"+PONG\r\n").await.unwrap();
            }
            "ECHO" => {
                let message = messages.get(4);
                if let Some(data) = message {
                    let size = data.len();
                    let reply_message = format!("${}\r\n{}\r\n", size, data);
                    stream.write(reply_message.as_bytes()).await.unwrap();
                }
            }
            "SET" => {
                let key_opt = messages.get(4);
                let value_opt = messages.get(6);

                match key_opt {
                    Some(k) => match value_opt {
                        Some(v) => {
                            let mut data = Data {
                                value: v.to_string(),
                                created_at: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis(),
                                expired_at: None,
                            };

                            let option_command_opt = messages.get(8);
                            println!("{:#?}", option_command_opt);
                            match option_command_opt {
                                Some(option_command) => {
                                    match option_command.to_uppercase().as_str() {
                                        "PX" => {
                                            let expired_at_opt = messages.get(10);
                                            match expired_at_opt {
                                                Some(exp) => {
                                                    let parsed = exp.parse::<u128>();
                                                    if !parsed.is_err() {
                                                        data.expired_at = Some(parsed.unwrap());
                                                    }
                                                }
                                                None => (),
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                None => (),
                            }

                            storage.insert(k.to_string(), data);
                            stream.write(b"+OK\r\n").await.unwrap();
                        }
                        None => {
                            stream.write(b"+FAIL\r\n").await.unwrap();
                        }
                    },
                    None => {
                        stream.write(b"+FAIL\r\n").await.unwrap();
                    }
                }
            }
            "GET" => {
                let key_opt = messages.get(4);
                let data_opt: Option<&Data> = match key_opt {
                    Some(k) => storage.get(*k),
                    None => None,
                };

                match data_opt {
                    Some(data) => {
                        if let Some(exp) = data.expired_at {
                            let current = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();

                            let expired = data.created_at + exp;
                            if current > expired {
                                storage.remove(*key_opt.unwrap());
                                stream.write(b"$-1\r\n").await.unwrap();
                                continue;
                            }
                        }

                        let size = data.value.len();
                        let reply_message = format!("${}\r\n{}\r\n", size, data.value);
                        stream.write(reply_message.as_bytes()).await.unwrap();
                    }
                    None => {
                        stream.write(b"$-1\r\n").await.unwrap();
                    }
                }
            }
            _ => {
                stream.write(b"+RECEIVED\r\n").await.unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process_stream(stream).await;
        });
    }
}
