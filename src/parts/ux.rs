
// src/Start_Workers
use super::time::now_in_utc;
use super::logger::return_to_owner;
use super::time::now_in_timestamp;
use super::database;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use database::{create_connection, update_user_info};
use teloxide::{prelude::*, utils::command::BotCommands, types::{CallbackQuery, Message, Update}, dispatching::UpdateFilterExt};
use std::collections::HashMap;
use anyhow::Context;
use chrono::{NaiveDateTime, Timelike};
use teloxide::payloads::SendMessageSetters;
use rusqlite::Connection;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId};
use crate::parts::database::{get_group_by_user_id};
use crate::parts::memcached::get_from_memcached;
use crate::parts::send_to_user::send_to_user_main;
use super::ring::{get_next_lesson, get_time_delta};
use crate::Secret::{TEST_BOT_TOKEN, PRODUCTION_BOT_TOKEN};

type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
type AsyncResult<T> = std::result::Result<T, BoxedError>;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show the groups")]
    Groups,
}


lazy_static::lazy_static! {
    static ref YEARS: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        map.insert("YEARS".to_string(), vec!["2024".to_string(),"2023".to_string(),"2022".to_string(),"2021".to_string(), ]);



        map
    };
    static ref DIRECTIONS: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        map.insert("2024".to_string(), vec!["24ВЕБ".to_string(),"24З".to_string(), "24ИИС".to_string(),
                             "24Л".to_string(), "24ОИБ".to_string(),"24П".to_string(),
                             "24ПД".to_string(),"24СА".to_string(),"24уКСК".to_string(),
                             "24уЛ".to_string(),"24Э".to_string(), "24Ю".to_string()]);


        map.insert("2023".to_string(), vec!["23ПО".to_string(), "23ВЕБ".to_string(),"23З".to_string(),
                             "23Л".to_string(), "23ОИБ".to_string(),"23П".to_string(),
                             "23ПД".to_string(),"23СА".to_string(),"23уКСК".to_string(),
                             "23уЛ".to_string(),"23Э".to_string()]);

        map.insert("2022".to_string(), vec!["22БД".to_string(), "22ВЕБ".to_string(), "22ЗИО".to_string() ,
                             "22ИС".to_string(), "22Л".to_string(),"22ОИБ".to_string(),
                             "22П".to_string(), "22ПД".to_string(), "22ПО".to_string(),
                             "22ПСА".to_string(), "22СА".to_string(), "22уКСК".to_string(),
                             "22Э".to_string()]);

        map.insert("2021".to_string(), vec!["21БД".to_string(), "21ВЕБ".to_string(), "21ЗИО".to_string(),
                             "21ИС".to_string(), "21Л".to_string(), "21ОИБ".to_string(),
                             "21П".to_string(), "21ПД".to_string(),  "21ПО".to_string(),
                             "21ПСА".to_string(), "21СА".to_string()
                             ,"21уКСК".to_string(), "21Э".to_string()]);
        map

    };
     static ref GROUPS: HashMap<String, Vec<String>> = {
        let mut map = HashMap::new();
        map.insert("24уКСК".to_string(), vec!["24УКСК-1".to_string(), "24УКСК-2".to_string()]);
        map.insert("24СА".to_string(), vec!["24СА-1".to_string(), "24СА-2".to_string(), "24СА-3".to_string()]);
        map.insert("24П".to_string(), vec!["24П-1".to_string(), "24П-2".to_string(), "24П-3".to_string(), "24П-4".to_string(), "24П-5".to_string()]);
        map.insert("24ВЕБ".to_string(), vec!["24ВЕБ-1".to_string(), "24ВЕБ-2".to_string()]);
        map.insert("24ИИС".to_string(), vec!["24ИИС-1".to_string(), "24ИИС-2".to_string()]);
        map.insert("24ОИБ".to_string(), vec!["24ОИБ-1".to_string(), "24ОИБ-2".to_string(), "24ОИБ-3".to_string()]);
        map.insert("24З".to_string(), vec!["24З-1".to_string(), "24З-2".to_string()]);
        map.insert("24Э".to_string(), vec!["24Э-1".to_string(), "24Э-2".to_string()]);
        map.insert("24Ю".to_string(), vec!["24Ю-1".to_string(), "24Ю-2".to_string(), "24Ю-3".to_string(), "24Ю-4".to_string()]);
        map.insert("24ПД".to_string(), vec!["24ПД-1".to_string(), "24ПД-2".to_string(), "24ПД-3".to_string()]);
        map.insert("24Л".to_string(), vec!["24Л-1".to_string(), "24Л-2".to_string()]);
        map.insert("24уЛ".to_string(), vec!["24УЛ-1".to_string()]);
        map.insert("23ВЕБ".to_string(), vec!["23ВЕБ-1".to_string(), "23ВЕБ-2".to_string()]);
        map.insert("23ПО".to_string(), vec!["23ПО-1".to_string(), "23ПО-2".to_string()]);
        map.insert("23СА".to_string(), vec!["23СА-1".to_string(), "23СА-2".to_string()]);
        map.insert("23уКСК".to_string(), vec!["23УКСК-1".to_string()]);
        map.insert("23уЛ".to_string(), vec!["23УЛ-1".to_string()]);
        map.insert("23Э".to_string(), vec!["23Э-1".to_string(), "23Э-2".to_string()]);
        map.insert("23П".to_string(), vec!["23П-1".to_string(), "23П-2".to_string(), "23П-3".to_string(), "23П-4".to_string(), "23П-5".to_string(), "23П-6".to_string()]);
        map.insert("23ПД".to_string(), vec!["23ПД-1".to_string(), "23ПД-2".to_string()]);
        map.insert("23ОИБ".to_string(), vec!["23ОИБ-1".to_string(), "23ОИБ-2".to_string()]);
        map.insert("23З".to_string(), vec!["23З-1".to_string(), "23З-2".to_string()]);
        map.insert("23Л".to_string(), vec!["23Л-1".to_string(), "23Л-2".to_string()]);
        map.insert("22БД".to_string(), vec!["22БД-1".to_string()]);
        map.insert("22ВЕБ".to_string(), vec!["22ВЕБ-1".to_string(), "22ВЕБ-2".to_string()]);
        map.insert("22ЗИО".to_string(), vec!["22ЗИО-1".to_string(), "22ЗИО-2".to_string()]);
        map.insert("22ИС".to_string(), vec!["22ИС-1".to_string()]);
        map.insert("22Л".to_string(), vec!["22Л-1".to_string(), "22Л-2".to_string()]);
        map.insert("22ОИБ".to_string(), vec!["22ОИБ-1".to_string(), "22ОИБ-2".to_string()]);
        map.insert("22П".to_string(), vec!["22П-1".to_string(), "22П-2".to_string(), "22П-3".to_string()]);
        map.insert("22ПД".to_string(), vec!["22ПД-1".to_string(), "22ПД-2".to_string()]);
        map.insert("22ПО".to_string(), vec!["22ПО-1".to_string(), "22ПО-2".to_string(), "22ПО-3".to_string()]);
        map.insert("22ПСА".to_string(), vec!["22ПСА-1".to_string(), "22ПСА-2".to_string(), "22ПСА-3".to_string()]);
        map.insert("22СА".to_string(), vec!["22СА-1".to_string(), "22СА-2".to_string()]);
        map.insert("22уКСК".to_string(), vec!["22УКСК-1".to_string(), "22УКСК-2".to_string()]);
        map.insert("22Э".to_string(), vec!["22Э-1".to_string(), "22Э-2".to_string()]);
       map.insert("21БД".to_string(), vec!["21БД-1".to_string()]);
        map.insert("21ВЕБ".to_string(), vec!["21ВЕБ-1".to_string(), "21ВЕБ-2".to_string()]);
        map.insert("21ЗИО".to_string(), vec!["21ЗИО-1".to_string(), "21ЗИО-2".to_string(), "21ЗИО-3".to_string()]);
        map.insert("21ИС".to_string(), vec!["21ИС-1".to_string()]);
        map.insert("21Л".to_string(), vec!["21Л-1".to_string(), "21Л-2".to_string()]);
        map.insert("21ОИБ".to_string(), vec!["21ОИБ-1".to_string(), "21ОИБ-2".to_string(), "21ОИБ-3".to_string()]);
        map.insert("21П".to_string(), vec!["21П-1".to_string(), "21П-2".to_string(), "21П-3".to_string()]);
        map.insert("21ПД".to_string(), vec!["21ПД-1".to_string(), "21ПД-2".to_string(), "21ПД-3".to_string()]);
        map.insert("21ПО".to_string(), vec!["21ПО-1".to_string(), "21ПО-2".to_string(), "21ПО-3".to_string(), "21ПО-4".to_string()]);
        map.insert("21ПСА".to_string(), vec!["21ПСА-1".to_string(), "21ПСА-2".to_string(), "21ПСА-3".to_string(), "21ПСА-4".to_string(), "21ПСА-6".to_string()]);
        map.insert("21СА".to_string(), vec!["21СА-1".to_string(), "21СА-2".to_string()]);
        map.insert("21уКСК".to_string(), vec!["21УКСК-1".to_string()]);
        map.insert("21Э".to_string(), vec!["21Э-1".to_string(), "21Э-2".to_string()]);
        map
    };
     static ref REPLACE_OPTIONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Изменить группу", "change_group_replace");
        m
    };

           static ref MAIN: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Настройки", "choice_setting_command");
        m.insert("Обновить", "get_replace");
        m.insert("До N-Пары", "get_time_to_N_lesson");
        m

    };

}
async fn start_message_with_update(bot: Bot, chat_id: ChatId, message_id: MessageId, timestamp: Instant) {
    bot.delete_message(chat_id, message_id).await.expect("Err in delete main message - str 151, UX");

    start_message(bot, chat_id, timestamp).await;
}
async fn start_message(bot: Bot, chat_id: ChatId, timestamp: Instant) {
    let group = get_group_by_user_id(chat_id.0);

    let keyboard = create_inline_keyboard(
        MAIN.iter().map(|(text, callback)| (text.to_string(), callback.to_string())).collect()
    );
    let replace = get_from_memcached(group).await.unwrap();
    let vec_time_to_lesson: Vec<String> = get_next_lesson().await.unwrap();
    let message_text: String = if vec_time_to_lesson.len() < 2 {

        let time_to_next_lesson = vec_time_to_lesson.get(0).map(|s| s.as_str()).unwrap_or("No lesson scheduled");

        format!("Главная:\n\n{}\n\nЗамены:\n{}", time_to_next_lesson, replace)
    } else {

        let time_to_next_lesson = vec_time_to_lesson.get(1).map(|s| s.as_str()).unwrap_or("No lesson scheduled");
        let time_to_next_lesson_with_launch = vec_time_to_lesson.get(0).map(|s| s.as_str()).unwrap_or("No lesson scheduled with launch");

        format!("Главная:\n\n{}\n\n{}\n\nЗамены:\n{}", time_to_next_lesson, time_to_next_lesson_with_launch, replace)
    };

    bot.send_message(chat_id, message_text)
        .reply_markup(keyboard)
        .await
        .with_context(|| format!("Failed to send message to chat ID: {}", chat_id)).unwrap();

    let end = Instant::now();

    let duration: std::time::Duration = end.duration_since(timestamp);
    if duration > std::time::Duration::from_millis(3000)  {
        // Что-то сделать, если продолжительность больше 3000 мс
        println!("Продолжительность {} мс превысила 3000 мс!", duration.as_millis());
        return_to_owner(duration);
    }

}
async fn start_command(bot: Bot, msg: Message) -> AsyncResult<()> {
    let chat_id = msg.chat.id;
    let timestamp = Instant::now();
    start_message(bot,chat_id, timestamp).await;
    Ok(())
}






async fn handle_callback_query(bot: Bot, q: CallbackQuery, conn: Arc<Mutex<Connection>>) -> AsyncResult<()> {
    if let Some(data) = &q.data {

        let chat_id: ChatId = q.message.as_ref().map(|m| m.chat.id).unwrap_or(q.from.id.into());
        println!("{}", chat_id.0);
        let message_id = q.message.as_ref().map(|m| m.id).unwrap_or(teloxide::types::MessageId(0));

        if data == "choice_setting_command"{
            let keyboard = create_inline_keyboard_with_back(
                REPLACE_OPTIONS.iter().map(|(text, callback)| (text.to_string(), callback.to_string())).collect(),
                "back_to_main".to_string()
            );
            bot.edit_message_text(chat_id, message_id, "Настройки")
                .reply_markup(keyboard)
                .await?;
        }

        else if data.starts_with("change_group_replace") {
            let year = data.strip_prefix("change_group_replace").unwrap();
            if let Some(choice_y) = YEARS.get("YEARS") {
                let keyboard = create_inline_keyboard_with_back(
                    choice_y.iter().map(|dir| (dir.to_string(), format!("year_{}", dir))).collect(),
                    "back_to_choice_replace_command".to_string()
                );
                bot.edit_message_text(chat_id, message_id, format!("Выберите год {}:", year))
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.send_message(chat_id, "Произошла ошибка: год не найден").await?;
            }

        } else if data.starts_with("year_") {
            let year = data.strip_prefix("year_").unwrap();
            if let Some(directions) = DIRECTIONS.get(year) {
                let keyboard = create_inline_keyboard_with_back(
                    directions.iter().map(|dir| (dir.to_string(), format!("direction_{}", dir))).collect(),
                    format!("change_group_replace{}", year)
                );
                bot.edit_message_text(chat_id, message_id, format!("Выбери направление для года {}:", year))
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.send_message(chat_id, "Произошла ошибка").await?;
            }
        } else if data.starts_with("direction_") {
            let direction = data.strip_prefix("direction_").unwrap();
            let year_ = DIRECTIONS.iter().find_map(|(y, dirs)| if dirs.contains(&direction.to_string()) { Some(y) } else { None }).map_or("", |v| v);
            if let Some(groups) = GROUPS.get(direction) {
                let keyboard = create_inline_keyboard_with_back(
                    groups.iter().map(|group| (group.to_string(), format!("group_{}", group))).collect(),
                    format!("year_{}", year_)
                );
                bot.edit_message_text(chat_id, message_id, format!("Выбери группу для направления {}:", direction))
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.send_message(chat_id, "Произошла ошибка").await?;
            }
        }
        else if data.starts_with("group_") {
            let group = match data.strip_prefix("group_") {
                Some(group) => group.to_lowercase(),
                None => {
                    bot.send_message(chat_id, "Неверный формат данных группы.").await?;
                    return Ok(());
                }
            };
            let keyboard = create_inline_keyboard(
                MAIN.iter().map(|(text, callback)| (text.to_string(), callback.to_string())).collect()
            );

            let user_id = q.from.id;
            let username = q.from.username.clone();
            let direction_ = GROUPS.iter().find_map(|(d, groups)| if groups.contains(&group) { Some(d) } else { None }).map_or("", |v| v);
            let _ = DIRECTIONS.iter().find_map(|(y, dirs)| if dirs.contains(&direction_.to_string()) { Some(y) } else { None }).map_or("", |v| v);

            update_user_info(user_id.0, group.to_string(), username, conn.clone()).await?;
            let chat_id = q.from.id;
            let group = get_group_by_user_id(chat_id.0 as i64);
            let replace = get_from_memcached(group).await.unwrap();
            let message_text = format!("Главная:\nЗамены \n {}", replace);

            bot.edit_message_text(chat_id, message_id, message_text)
                .reply_markup(keyboard)
                .await
                .with_context(|| format!("Failed to send message to chat ID: {}", chat_id))?;

            println!("Message sending (UX)");




        } else if data == "get_time_to_N_lesson" {
            // Create a list of numbers to display in the keyboard
            let number_buttons: Vec<(String, String)> = (1..=7) // Assuming 7 lessons max
                .map(|i| (i.to_string(), format!("lesson_number_{}", i)))
                .collect();

            let keyboard = create_inline_keyboard(number_buttons);

            bot.edit_message_text(chat_id, message_id, "Выберите номер пары:")
                .reply_markup(keyboard)
                .await?;
        } else if data.starts_with("lesson_number_") {
            let lesson_number_str = data.strip_prefix("lesson_number_").unwrap();
            match lesson_number_str.parse::<u32>() {
                Ok(lesson_number) => {
                    // Call your function to process the lesson number

                    match get_time_delta(lesson_number).await {
                        Ok(result_message) => {
                            let timestamp  = Instant::now();

                            start_message_with_update(bot.clone(), chat_id, message_id, timestamp).await;

                            send_to_user_main(result_message, chat_id.0).await;
                        }
                        Err(error_message) => {
                            bot.send_message(chat_id, error_message).await;
                        }
                    }
                    bot.answer_callback_query(q.id).await; // Подтверждаем получение callback
                }
                Err(_) => {
                    bot.send_message(chat_id, "Некорректный номер пары.").await?;
                }
            }


        } else if data.starts_with("get_replace") {
            let timestamp  = Instant::now();

            start_message_with_update(bot, chat_id, message_id, timestamp).await;

        } else if data == "back_to_main" {
            let timestamp  = Instant::now();
            start_message_with_update(bot, chat_id, message_id, timestamp).await;
        } else if data == "back_to_choice_replace_command"{
            let keyboard = create_inline_keyboard_with_back(
                REPLACE_OPTIONS.iter().map(|(text, callback)| (text.to_string(), callback.to_string())).collect(),
                "back_to_main".to_string()
            );
            bot.edit_message_text(chat_id, message_id, "Выберите действие с заменами:")
                .reply_markup(keyboard)
                .await?;
        }
    }
    Ok(())
}

fn create_inline_keyboard(buttons: Vec<(String, String)>) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = buttons
        .chunks(2)
        .map(|chunk| {
            chunk
                .iter()
                .map(|(text, callback)| InlineKeyboardButton::callback(text.to_string(), callback.to_string()))
                .collect()
        })
        .collect();
    InlineKeyboardMarkup::new(keyboard)
}
fn create_inline_keyboard_with_back(buttons: Vec<(String, String)>, back_callback: String) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = buttons
        .chunks(2)
        .map(|chunk| {
            chunk
                .iter()
                .map(|(text, callback)| InlineKeyboardButton::callback(text.to_string(), callback.to_string()))
                .collect()
        })
        .collect();
    // Добавляем кнопку "Назад"
    keyboard.push(vec![InlineKeyboardButton::callback("Назад".to_string(), back_callback)]);
    InlineKeyboardMarkup::new(keyboard)
}



async fn run() -> AsyncResult<()> {
    let bot_token: &str  = PRODUCTION_BOT_TOKEN;
    let bot = Bot::new(bot_token);
    let db_path = "Database.db";
    let conn = Arc::new(Mutex::new(create_connection(db_path)?));
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(
                    |bot: Bot, msg: Message, cmd: Command| async move {
                        match cmd {
                            Command::Start => start_command(bot, msg).await,
                            Command::Groups => start_command(bot, msg).await,
                        }
                    }
                )
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(
                    |bot: Bot, callback_query: CallbackQuery, conn: Arc<Mutex<Connection>>| async move {
                        handle_callback_query(bot, callback_query, conn).await
                    }
                )
        );


    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![conn])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}




pub(crate) async fn start_ux() {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    if let Err(err) = run().await {
        log::error!("Error running bot: {}", err);
    }
}