use snapshot::Snapshot;

pub mod snapshot;

pub const DEFAULT_ADDR: &str = "127.0.0.1:6379";
pub const FLAG_DIR: &str = "dir";
pub const FLAG_DBFILENAME: &str = "dbfilename";
pub const FLAG_ADDR: &str = "addr";

#[derive(Clone, Debug)]
pub struct State {
    pub snapshot: Snapshot,
}

impl State {
    pub fn new(directory: Option<String>, filename: Option<String>) -> State {
        State {
            snapshot: Snapshot::new(
                directory.unwrap_or("./tmp".to_string()),
                filename.unwrap_or("snapshot.rdb".to_string()),
            ),
        }
    }
}
