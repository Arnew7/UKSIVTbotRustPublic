
use std::time::Duration;
use teloxide::Bot;
use teloxide::types::{ChatId as TeloxideChatId, MessageId as TeloxideMessageId}; // Переименовываем, чтобы избежать конфликта
use tokio::time::sleep;
use crate::parts::database::{get_user_and_group_and_message_id, get_user_and_group};
use crate::parts::memcached::get_from_memcached;
use crate::parts::send_to_user::send_to_user_main;
use super::replace::replacements_main;
use super::ux::start_message_with_update;
use crate::Secret::{TEST_BOT_TOKEN, PRODUCTION_BOT_TOKEN};



pub async fn cycle_work_replace() {
    loop {
        match replacements_main().await {
            Ok(_) => {}
            Err(_) => {}
        };
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
// Структура для хранения информации для удаления сообщений.
#[derive(Debug)]
pub struct Info_for_del{
    pub id: TeloxideChatId,
    pub Message_id: TeloxideMessageId,
}

// Асинхронная функция для отправки уведомлений.
pub async fn send_notification() -> anyhow::Result<()> {
    tokio::time::sleep(Duration::from_secs(15)).await;
    let bot_token: &str  = TEST_BOT_TOKEN;

    let info = get_user_and_group_and_message_id().unwrap();

    for user in info {
        let chat_id = user.id;
        let message_id = user.Message_id;
        let bot = Bot::new(bot_token);

        let teloxide_chat_id = TeloxideChatId(chat_id.0);
        let teloxide_message_id= TeloxideMessageId(message_id.0);

        start_message_with_update(bot, teloxide_chat_id, teloxide_message_id).await;
    }

    Ok(())
}
