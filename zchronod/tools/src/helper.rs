use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_time_ms() -> u128 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time
}