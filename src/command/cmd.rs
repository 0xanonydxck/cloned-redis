use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Command {
    pub command: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub option: HashMap<String, String>,
}

impl Command {
    pub fn build(message: String) -> Result<Command, ErrorCommand> {
        let mut parts: Vec<&str> = message.split("\r\n").collect();
        parts.remove(parts.len() - 1); // remove the rest of empty chars (x00\r\n)

        let mut cmd = Command {
            command: parts.get(2).unwrap().to_uppercase(),
            key: None,
            value: None,
            option: HashMap::new(),
        };

        match cmd.command.as_str() {
            SET_ROUTER => {
                /*
                    `SET` command structure must include:
                    [*x, $x, cmd, $x, key, $x, value]
                */
                if parts.len() < 7 {
                    return Err(ErrorCommand::InvalidInput);
                } else {
                    cmd.key = Some(parts.get(4).unwrap().to_string());
                    cmd.value = Some(parts.get(6).unwrap().to_string());
                }

                if parts.len() > 7 {
                    let opt_parts = &parts[7..];
                    if opt_parts.len() % 4 == 0 {
                        let mut key_index: usize = 8;
                        let mut value_index: usize = 10;

                        for i in 0..opt_parts.len() / 4 {
                            key_index = key_index + (4 * i);
                            value_index = value_index + (4 * i);
                            cmd.option.insert(
                                parts[key_index].to_uppercase(),
                                parts[value_index].to_string(),
                            );
                        }
                    }
                }
            }
            _ => {
                /*
                    other command structure must include:
                    [*x, $x, cmd, $x, value]
                */
                cmd.value = match parts.get(4) {
                    Some(v) => Some(v.to_string()),
                    None => None,
                };

                if parts.len() > 5 {
                    let opt_parts = &parts[5..];
                    if opt_parts.len() % 4 == 0 {
                        let mut key_index: usize = 8;
                        let mut value_index: usize = 10;

                        for i in 0..opt_parts.len() / 4 {
                            key_index = key_index + (4 * i);
                            value_index = value_index + (4 * i);
                            cmd.option.insert(
                                parts[key_index].to_uppercase(),
                                parts[value_index].to_string(),
                            );
                        }
                    }
                }
            }
        }

        Ok(cmd)
    }
}

#[derive(Debug)]
pub enum ErrorCommand {
    InvalidInput,
}
