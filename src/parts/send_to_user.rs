use std::error::Error;
use teloxide::{prelude::*, requests::Requester};
use teloxide::types::ChatId;
use std::thread::sleep;
use std::time;
use crate::Secret::{TEST_BOT_TOKEN, ERR_BOT_TOKEN, PRODUCTION_BOT_TOKEN};
async fn send_telegram_message(bot: Bot, data: String, chat_id: i64) -> ResponseResult<()> {
    let chat_id = ChatId(chat_id);
    let message = bot.send_message(chat_id, data).await?;
    let message_id = message.id;
    sleep(time::Duration::from_secs(10));
    let _ = bot.delete_message(chat_id, message_id).await;

    Ok(())
}

async fn send_telegram_message_without_delete(bot: Bot, data: String, chat_id: i64) -> ResponseResult<()> {
    let chat_id = ChatId(chat_id);
    let message = bot.send_message(chat_id, data).await?;
    let message_id = message.id;


    Ok(())
}




pub async fn send_to_user_main(data: String, chat_id: i64) -> Result<(), Box<dyn Error>> {
    let bot_token: &str  =  PRODUCTION_BOT_TOKEN;

    let bot = Bot::new(bot_token);

    match send_telegram_message(bot, data, chat_id).await {
        Ok(_) => {
            println!("Сообщение успешно отправлено!");
            Ok(())
        },
        Err(err) => {
            eprintln!("Ошибка отправки сообщения: {}", err);
            Err(err.into())
        },
    }
}

pub async fn send_to_owner(data: String, chat_id: i64) -> Result<(), Box<dyn Error>> {
    let bot_token: &str  =  ERR_BOT_TOKEN;

    let bot = Bot::new(bot_token);

    match send_telegram_message_without_delete(bot, data, chat_id).await {
        Ok(_) => {
            println!("Сообщение успешно отправлено!");
            Ok(())
        },
        Err(err) => {
            eprintln!("Ошибка отправки сообщения: {}", err);
            Err(err.into())
        },
    }
}


