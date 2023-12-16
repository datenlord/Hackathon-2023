use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> u128 {
    let current_time = SystemTime::now();
    let duration = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get current time");
    let milliseconds = duration.as_millis();

    return milliseconds;
}
