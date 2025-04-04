
extern crate rusqlite;

use rusqlite::{Connection, Result};

use std::env;

pub(crate) async fn create_database() -> Result<()> {
    let current_dir = env::current_dir();
    println!("Current working directory: {:?}", current_dir);

    let conn = Connection::open("Database.db")?; //Attempt to open database.  Error will be returned if it fails here.

    conn.execute(
        "CREATE TABLE IF NOT EXISTS info_users (
            user_id INTEGER PRIMARY KEY,
            group_inf TEXT DEFAULT '23веб-1',
            username TEXT DEFAULT NULL,
            save_time INTEGER DEFAULT 30000 -- 5:00 в секундах
        )",
        [],
    )?;

    println!("Database and tables created successfully.");
    Ok(())
}
