use std::str::FromStr;
use std::time::Duration;
use teloxide::types::{ChatId as TeloxideChatId, MessageId as TeloxideMessageId};
use teloxide::Bot;
use super::replace::replacements_main;
use super::ux::start_message_with_update;
use crate::parts::database::get_user_and_message_id_by_group;
use crate::parts::memcached::{get_from_memcached, write_on_memcached};
use crate::Secret::{GROUPS_VEC, PRODUCTION_BOT_TOKEN, TEST_BOT_TOKEN};


pub async fn cycle_work_replace() {
    loop {
        match replacements_main().await {
            Ok(_) => {}
            Err(e) => {
                println!("Error in replacements_main: {:?}", e);
            }
        };
        tokio::time::sleep(Duration::from_secs(60)).await;
        send_notification().await.expect("Ошибка при автоматической отправке новых замен from Cycle str 22");
    }
}
// Структура для хранения информации для удаления сообщений.
#[derive(Debug)]
pub struct InfoForDel {
    pub id: TeloxideChatId,
    pub Message_id: TeloxideMessageId,
}

pub async fn send_notification() -> Result<(), anyhow::Error> {
    let bot_token: &str  = PRODUCTION_BOT_TOKEN;
    let bot = Bot::new(bot_token);

    let mut groups_to_notify: Vec<String> = Vec::new();

    for group in GROUPS_VEC.clone() {

        let group_weight_key = format!("{}_weight", group);

        let weight_now = get_from_memcached(group.clone()).await?.len();
        let weight_last_str = get_from_memcached(group_weight_key.clone()).await?;
        let weight_last: usize = usize::from_str(&weight_last_str).unwrap_or(0);


        if weight_now == weight_last {
            continue;
        }
        groups_to_notify.push(group);
        write_on_memcached(weight_now.to_string(), group_weight_key.clone()).await.expect("Ошибка при записи в кэш from Cycle str 49");
    }

    for group_name in groups_to_notify {
        let info = get_user_and_message_id_by_group(group_name.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Ошибка получения user, group and message_id: {}", e))?;

        for user in info {
            let chat_id = user.id;
            let message_id = user.Message_id;

            let teloxide_chat_id = TeloxideChatId(chat_id.0);
            let teloxide_message_id = TeloxideMessageId(message_id.0);

            start_message_with_update(bot.clone(), teloxide_chat_id, teloxide_message_id).await;

        }
    }

    Ok(())
}
