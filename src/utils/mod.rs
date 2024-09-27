use std::{collections::HashMap, env};

pub fn parse_flag(keys: Vec<String>) -> HashMap<String, String> {
    let args: Vec<String> = env::args().collect();
    let mut result: HashMap<String, String> = HashMap::new();

    for key in keys {
        match args.binary_search(&format!("--{}", key)) {
            Ok(idx) => {
                if idx + 1 < args.len() && !args[idx + 1].contains("--") {
                    result.insert(key, args.get(idx + 1).unwrap().clone());
                }
            }
            Err(_) => (),
        }
    }

    result
}
