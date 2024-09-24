use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CacheData {
    pub value: Vec<u8>,
    pub timestamp: u128,        // unix timestmap in milliseconds unit
    pub lifetime: Option<u128>, // milliseconds unit
}

impl CacheData {
    pub fn is_expired(self) -> bool {
        match self.lifetime {
            Some(lifetime) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                let expired_time = self.timestamp + lifetime;
                if now > expired_time {
                    return true;
                }

                false
            }
            None => false,
        }
    }
}
