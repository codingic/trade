pub fn backfill_start_ms(days: u64) -> u64 {
    now_ms().saturating_sub(days * 24 * 3600 * 1000)
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let offset = 8 * 3600;
    let local = now + offset;
    let (h, m, s) = (
        (local % 86400) / 3600,
        (local % 3600) / 60,
        local % 60,
    );
    format!("{h:02}:{m:02}:{s:02}")
}