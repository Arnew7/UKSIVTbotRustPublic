use anyhow::{Result, Context};
use sqlx::{FromRow};
use crate::parts::db::db_pool::get_pool;

#[derive(Debug, FromRow)]
pub struct UserMessageId {
    pub id: i64,
    pub message_id: Option<i32>,
}

pub async fn update_user_info(chat_id: i64, group_inf: String) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO info_users (chat_id, group_inf) VALUES (?1, ?2)")
        .bind(chat_id)
        .bind(group_inf)

        .execute(get_pool())
        .await
        .context("Ошибка при обновлении информации о пользователе")?;

    Ok(())
}

pub async fn update_user_message_id(chat_id: i64, message_id: i32) -> Result<()> {
    sqlx::query(
        "UPDATE info_users SET message_id = ? WHERE chat_id = ?")
        .bind(message_id)
        .bind(chat_id)
        .execute(get_pool())
        .await
        .context("Ошибка при обновлении ID последнего сообщения")?;

    Ok(())
}

pub async fn get_user_and_message_id_by_group(group_inf: String) -> Result<Vec<UserMessageId>> {
    let result = sqlx::query_as::<_, UserMessageId>(
        "SELECT chat_id as id, message_id FROM info_users WHERE group_inf = ?1",
    )
        .bind(group_inf)
        .fetch_all(get_pool())
        .await
        .context("Ошибка при получении пользователей по группе")?;

    Ok(result)
}

pub async fn get_group_by_chat_id(chat_id: i64) -> Result<String> {
    let result = sqlx::query_scalar(
        "SELECT group_inf FROM info_users WHERE chat_id = ?1",
    )
        .bind(chat_id)
        .fetch_optional(get_pool())
        .await;

    match result {
        Ok(Some(group)) => Ok(group),
        Ok(None) => Ok("23веб-1".to_string()),
        Err(err) => {
            eprintln!("Ошибка при получении группы: {:?}", err);
            Ok("23веб-1".to_string())
        }
    }
}

