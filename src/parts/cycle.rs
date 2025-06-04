use super::db::db_instant::DB;
use super::replace::replacements_main;
use super::ux::start_message_with_update;
use crate::parts::cache;
use crate::parts::cache::CacheInterface;
use crate::parts::db::interface_db::InterfaceDB;
use crate::Secret::{get_bot, GROUPS_VEC};
use anyhow::anyhow;
use futures::TryFutureExt;
use std::time::Duration;
use teloxide::types::{ChatId as TeloxideChatId, ChatId, MessageId as TeloxideMessageId, MessageId};


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
    pub message_id: TeloxideMessageId,
}

use chrono::{Local, Timelike};

pub async fn send_notification() -> Result<(), anyhow::Error> {
    let bot = get_bot();
    let cache = cache::MemcachedCache::new();

    let now = Local::now();
    let hour = now.hour();
    let minute = now.minute();

    let in_night_window = (hour == 23 && minute >= 59) || (hour == 0 && minute <= 5);

    let mut groups_to_notify: Vec<String> = Vec::new();

    for group in GROUPS_VEC.clone() {
        let group_weight_key = format!("{}_weight", group);

        // Получаем текущий вес (вес - длина в байтах значения)
        let weight_now = match cache.get(&group).await? {
            Some(value) => value.len(),
            None => 0,
        };

        // Получаем вес из кеша, который хранится как usize в 8 байтах BE
        let weight_last = match cache.get(&group_weight_key).await? {
            Some(bytes) if bytes.len() == 8 => {
                let arr: [u8; 8] = bytes.as_slice().try_into()?;
                usize::from_be_bytes(arr)
            },
            _ => 0,
        };

        if weight_now == weight_last {
            continue; // вес не изменился, пропускаем группу
        }

        // Всегда обновляем вес, даже если уведомление не будет отправлено
        let weight_bytes = weight_now.to_be_bytes();
        cache.set(&group_weight_key, &weight_bytes, 0).await
            .map_err(|e| anyhow!("Ошибка при записи в кэш: {}", e))?;

        // Если сейчас ночь, не добавляем в очередь на уведомление
        if !in_night_window {
            groups_to_notify.push(group.clone());
        }
    }

    // Если "ночное окно", уведомления не отправляем
    if in_night_window {
        return Ok(());
    }

    for group_name in groups_to_notify {
        let info = DB.get_user_and_message_id_by_group(group_name.clone())
            .await
            .map_err(|e| anyhow!("Ошибка получения user, group и message_id: {}", e))?;

        for user in info {
            let chat_id = user.id;
            let message_id = user.message_id.unwrap_or(0);

            let teloxide_chat_id = ChatId(chat_id);
            let teloxide_message_id = MessageId(message_id);

            start_message_with_update(bot.clone(), teloxide_chat_id, teloxide_message_id).await;


        }
    }

    Ok(())
}


