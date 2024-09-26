pub struct State {
    pub directory: String,
    pub filename: String,
}

impl State {
    pub fn new(directory: Option<String>, filename: Option<String>) -> State {
        State {
            directory: directory.unwrap_or("/tmp".to_string()),
            filename: filename.unwrap_or("snapshot.rdb".to_string()),
        }
    }
}
