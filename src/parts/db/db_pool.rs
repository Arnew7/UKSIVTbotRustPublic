use once_cell::sync::OnceCell;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use anyhow::Result;

static DB_POOL: OnceCell<SqlitePool> = OnceCell::new();

pub async fn init_db() -> Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:Database.db")
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS info_users (
            chat_id INTEGER PRIMARY KEY,
            group_inf TEXT DEFAULT '23веб-1',
            message_id INTEGER DEFAULT NULL
        )"
    )
        .execute(&pool)
        .await?;

    DB_POOL.set(pool).expect("DB_POOL was already set");
    Ok(())
}

pub fn get_pool() -> &'static SqlitePool {
    DB_POOL.get().expect("DB pool is not initialized")
}
