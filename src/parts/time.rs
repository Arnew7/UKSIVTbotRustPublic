
use chrono::{Local, Duration, Utc, Timelike, FixedOffset};
use chrono_tz::TzOffset;
use tokio::task;

/// Returns today's day in DD format.
pub async fn today() -> String {
    task::spawn_blocking(|| Local::now().format("%d.%m").to_string()).await.unwrap()
}


/// Returns tomorrow's day in DD format.
pub async fn tomorrow() -> String {
    task::spawn_blocking(|| (Local::now() + Duration::days(1)).format("%d.%m").to_string()).await.unwrap()
}

/// Returns the day after tomorrow in DD format.
pub async fn after_tomorrow() -> String {
    task::spawn_blocking(|| (Local::now() + Duration::days(2)).format("%d.%m").to_string()).await.unwrap()
}



pub async fn now_in_utc() -> (u32, u32) {
    let now_uf = Utc::now();
    let timestamp = now_uf.timestamp();
    // Возвращаем время в формате UTC
    let hours = now_uf.hour();
    let minutes = now_uf.minute();

    // Возвращаем кортеж с часами и минутами
    (hours, minutes)

}

pub async fn now_in_timestamp() -> i64 {
    let now_yekat = Utc::now().with_timezone(&FixedOffset::east(5 * 3600));
    let timestamp = now_yekat.timestamp();

    timestamp
}


