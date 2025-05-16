use std::time::Duration;
use teloxide::types::{ChatId as TeloxideChatId, MessageId as TeloxideMessageId};
use teloxide::Bot;
use super::replace::replacements_main;
use super::ux::start_message_with_update;
use crate::parts::database::get_user_and_group_and_message_id;
use crate::Secret::{PRODUCTION_BOT_TOKEN, TEST_BOT_TOKEN};


pub async fn cycle_work_replace() {
    loop {
        match replacements_main().await {
            Ok(_) => {}
            Err(_) => {}
        };
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
// Структура для хранения информации для удаления сообщений.
#[derive(Debug)]
pub struct InfoForDel {
    pub id: TeloxideChatId,
    pub Message_id: TeloxideMessageId,
}

// Асинхронная функция для отправки уведомлений.
pub async fn send_notification() -> anyhow::Result<()> {
    tokio::time::sleep(Duration::from_secs(15)).await;
    let bot_token: &str  = PRODUCTION_BOT_TOKEN;
    println!("Файлы изменились");
    let info = get_user_and_group_and_message_id().expect("Ошибка получинии user, group and message_id from Cycle str 36");

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
