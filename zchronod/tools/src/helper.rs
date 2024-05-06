use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Sha256, Digest};

pub fn get_time_ms() -> u128 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time
}

pub fn sha256_str_to_hex(fclock_str: String) -> String {
    let mut f_hasher = Sha256::new();
    f_hasher.update(fclock_str.clone());
    let f_hash_str = f_hasher.finalize();
    let f_hash_hex = format!("{:x}", f_hash_str);
    f_hash_hex
}