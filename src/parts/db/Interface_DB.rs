use async_trait::async_trait;
use anyhow::Result;
use crate::parts::db::database::{self, UserMessageId};

#[async_trait]
pub trait InterfaceDB: Send + Sync {
    async fn update_user_info(&self, chat_id: i64, group_inf: String) -> Result<()>;
    async fn update_user_message_id(&self, chat_id: i64, message_id: i32) -> Result<()>;
    async fn get_user_and_message_id_by_group(&self, group_inf: String) -> Result<Vec<UserMessageId>>;
    async fn get_group_by_chat_id(&self, chat_id: i64) -> Result<String>;
}

#[derive(Clone)]
pub struct GlobalDB;

#[async_trait]
impl InterfaceDB for GlobalDB {
    async fn update_user_info(&self, chat_id: i64, group_inf: String) -> Result<()> {
        database::update_user_info(chat_id, group_inf).await
    }

    async fn update_user_message_id(&self, chat_id: i64, message_id: i32) -> Result<()> {
        database::update_user_message_id(chat_id, message_id).await
    }

    async fn get_user_and_message_id_by_group(&self, group_inf: String) -> Result<Vec<UserMessageId>> {
        database::get_user_and_message_id_by_group(group_inf).await
    }

    async fn get_group_by_chat_id(&self, chat_id: i64) -> Result<String> {
        database::get_group_by_chat_id(chat_id).await
    }
}
