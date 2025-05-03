
use std::error::Error;
use std::path::Path;
use bytes::Bytes;
use std::sync::Arc;
use teloxide::{prelude::*, types::InputFile};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use chrono::{NaiveDateTime, TimeZone};
use rusqlite::{params, Connection, Error as RusqliteError, Result, Row};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use teloxide::types::{ChatId as TeloxideChatId, MessageId as TeloxideMessageId, MessageId};
use tokio::sync::Mutex;
use std::convert::TryFrom;
use FromSqlError::OutOfRange;

// Тип для хранения подключения к базе данных, обернутый в Arc<Mutex<>> для безопасного доступа из разных потоков.
type UserRepo = Arc<Mutex<Connection>>;

// Удобный тип для возврата результатов асинхронных операций с обработкой ошибок.
type AsyncResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub fn create_connection(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS info_users (
            chat_id INTEGER PRIMARY KEY,
            group_inf TEXT DEFAULT '23веб-1',
            username TEXT DEFAULT NULL,
            message_id INTEGER DEFAULT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub async fn update_user_info(chat_id: i64, group: String, username: Option<String>, conn: UserRepo) -> Result<()> {
    let conn = conn.lock().await; // Получаем мьютекс для эксклюзивного доступа к базе данных.
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO info_users (chat_id, group_inf, username) VALUES (?, ?, ?)",
    )?;
    stmt.execute(params![chat_id, group, username])?; // Выполняем запрос с параметрами.
    Ok(())
}

pub async fn update_user_message_id(chat_id: ChatId, message_id: MessageId) -> Result<()> {
    let db_path = "Database.db";
    let conn = Arc::new(Mutex::new(create_connection(db_path)?)); // Создаем подключение к базе данных.
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO info_users (chat_id, message_id) VALUES (?, ?)",
    )?;
    stmt.execute(params![chat_id.0, message_id.0])?; // Используем .0 для извлечения i64
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DatabaseChatId (pub i64);

// Реализация FromSql для DatabaseChatId, чтобы можно было извлекать значения из базы данных.
impl FromSql for DatabaseChatId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(i) => Ok(DatabaseChatId(i)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType), // Обработка неожиданного типа
        }
    }
}

// Структура для хранения message_id из базы данных.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DatabaseMessageId(pub i32);

impl FromSql for DatabaseMessageId {
    fn column_result(value: ValueRef<'_>) -> Result<DatabaseMessageId, FromSqlError> {
        match value {
            ValueRef::Integer(i) => {

                match i32::try_from(i) {
                    Ok(val) => Ok(DatabaseMessageId(val)),

                    Err(_) => Err(FromSqlError::OutOfRange(i)),
                }
            }
            _ => Err(FromSqlError::InvalidType), // Handle unexpected type from DB
        }
    }
}
// Структура для хранения информации о пользователе, группе и message_id.
#[derive(Debug)]
pub struct User_Message_id {
    pub id: DatabaseChatId,
    pub group: String,
    pub Message_id: DatabaseMessageId,
}

// Функция для получения списка User_Message_id из базы данных.
pub fn get_user_and_group_and_message_id() -> Result<Vec<User_Message_id>, RusqliteError> {
    let conn = Connection::open("Database.db")?;
    let mut stmt = conn.prepare("SELECT chat_id, group_inf ,Message_id FROM info_users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User_Message_id {
            id: row.get(0)?,
            group: row.get(1)?,
            Message_id: row.get(2)?,
        })
    })?;

    let mut users: Vec<User_Message_id> = Vec::new();
    for user_result in user_iter {
        let user = user_result?;
        users.push(user);
    }

    Ok(users)
}

pub async fn reg_user_info(chat_id: i64, username: Option<String>, conn: UserRepo) -> Result<()> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO info_users (chat_id, username) VALUES (?, ?)",
    )?;
    stmt.execute(params![chat_id, username])?;
    Ok(())
}

// Функция для получения группы пользователя по chat_id.
pub fn get_group_by_chat_id(chat_id: i64) -> String {
    match Connection::open("Database.db") {
        Ok(conn) => {
            match conn.query_row(
                "SELECT group_inf FROM info_users WHERE chat_id = ?",
                params![chat_id],
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

// Структура для хранения информации о пользователе (id, имя, группа).
#[derive(Debug)]
pub struct User {
    id: i64,
    name: String,
    group: String,
}

// Функция для получения списка всех пользователей из базы данных.
pub fn get_all_users() -> Result<Vec<User>> {
    let conn = Connection::open("Database.db")?;
    let mut stmt = conn.prepare("SELECT chat_id, group_inf, username FROM info_users")?;
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

// Структура для хранения информации о пользователе и его группе.
#[derive(Debug)]
pub struct User_Group {
    pub id: i64,
    pub group: String,
}

// Функция для получения списка User_Group из базы данных.
pub fn get_user_and_group() -> Result<Vec<User_Group>, RusqliteError> {
    let conn = Connection::open("Database.db")?;
    let mut stmt = conn.prepare("SELECT chat_id, group_inf FROM info_users")?;
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
