use chrono::{DateTime, FixedOffset, TimeZone, Utc};

pub fn format_timestamp(timestamp_ms: i64, format: &str) -> String {
    let dt_utc = Utc.timestamp_millis_opt(timestamp_ms).unwrap();
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let dt_bj: DateTime<FixedOffset> = dt_utc.with_timezone(&tz);

    let formatted = dt_bj.format(format).to_string();
    return formatted;
}

pub fn format_now_with_diff(format: &str, diff: i64) -> String {
    let ts = Utc::now().timestamp_millis() + diff;
    return format_timestamp(ts, format);
}


