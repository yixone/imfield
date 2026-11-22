use chrono::Utc;

pub fn get_now_ms() -> i64 {
    Utc::now().timestamp_millis()
}
