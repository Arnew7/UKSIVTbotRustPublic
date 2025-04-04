use std::error::Error;
use std::path::Path;
use bytes::Bytes;

use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use chrono::{NaiveDateTime, TimeZone};
use rusqlite::{params, Connection, Error as RusqliteError, Result};
use tokio::sync::Mutex;
type UserRepo = Arc<std::sync::Mutex<Connection>>;
type AsyncResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
// Функция для создания подключения к базе данных
pub fn create_connection(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS info_users (
            user_id INTEGER PRIMARY KEY,
            group_inf TEXT DEFAULT '23веб-1',
            username TEXT DEFAULT NULL,
            save_time INTEGER DEFAULT 30000 -- 5:00 в секундах
        )",
        [],
    )?;
    Ok(conn)
}






// Асинхронная функция для обновления данных пользователя
pub async fn update_user_info(user_id: u64, group: String, username: Option<String>, conn: Arc<Mutex<Connection>>,) -> Result<()> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO info_users (user_id, group_inf, username) VALUES (?, ?, ?)",
    )?;
    stmt.execute(params![user_id, group, username])?;
    Ok(())
}

pub async fn reg_user_info(user_id: u64, username: Option<String>, conn: Arc<Mutex<Connection>>,) -> Result<()> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO info_users (user_id, username) VALUES (?, ?)",
    )?;
    stmt.execute(params![user_id, username])?;
    Ok(())
}

pub fn get_group_by_user_id(user_id: i64) -> String {
    match Connection::open("Database.db") {
        Ok(conn) => {
            match conn.query_row(
                "SELECT group_inf FROM info_users WHERE user_id = ?",
                params![user_id],
                |row| row.get::<_, String>(0),
            ) {
                Ok(group) => group,
                Err(err) => {
                    eprintln!("Error retrieving group: {}", err);
                    "23веб-1".to_string()
                }
            }
        }
        Err(err) => {
            eprintln!("Error opening database: {}", err);
            "23веб-1".to_string()
        }
    }
}


#[derive(Debug)]
pub struct User {
    id: i64,
    name: String,
    group: String,
}

pub fn get_all_users() -> Result<Vec<User>> {
    let conn = Connection::open("Database.db")?;
    let mut stmt = conn.prepare("SELECT user_id, group_inf, username FROM info_users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            group: row.get(1)?,
            name: row.get(2)?,
        })
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        let user = user?;
        println!("{:?}", user);
        users.push(user);
    }

    Ok(users)
}



#[derive(Debug)]
pub struct User_Group {
    pub id: i64,
    pub group: String,
}

pub fn get_user_and_group() -> Result<Vec<User_Group>, RusqliteError> {
    let conn = Connection::open("Database.db")?;
    let mut stmt = conn.prepare("SELECT user_id, group_inf FROM info_users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User_Group {
            id: row.get(0)?,
            group: row.get(1)?,
        })
    })?;

    let mut users: Vec<User_Group> = Vec::new();
    for user_result in user_iter {
        let user = user_result?;
        users.push(user);
    }

    Ok(users)
}


