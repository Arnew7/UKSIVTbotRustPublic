use std::time::Duration;
use tokio::time::sleep;
use crate::parts::database::get_user_and_group;
use crate::parts::memcached::get_from_memcached;
use crate::parts::send_to_user::send_to_user_main;
use super::replace::replacements_main;

pub async fn cycle_work_replace() {
    loop {
        match replacements_main().await {
            Ok(_) => {}
            Err(_) => {}
        };
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}


pub async fn send_notification(message: String) -> anyhow::Result<()> {
    tokio::time::sleep(Duration::from_secs(15)).await;
    let users = get_user_and_group()?;

    // 2. Итерироваться по пользователям и отправлять им сообщения
    for user in users {
        let chat_id = user.id;
        let group = user.group;

        // 3. Получить сообщение из Memcached для этой группы
        match get_from_memcached(group).await {
            Ok(message) => {
                // 4. Отправить сообщение пользователю

                send_to_user_main(message, chat_id).await; // Используем существующую функцию
            }
            Err(e) => {
                //  Просто продолжаем цикл
            }
        }
    }

    Ok(())
}
