mod parts;

mod Secret;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::time::Duration;
use teloxide::Bot;
use tokio::{select, task, time};
use tracing::subscriber::set_global_default;
use futures::TryFutureExt;
use crate::parts::cache;
use crate::parts::cache::CacheInterface;
use crate::Secret::{MEMCACHED_PRODUCTION_ADDRESS, GROUPS_VEC};
use chrono::NaiveDate;
use parts::excel::models::Student;
use parts::excel::config::ReportHeader;
use parts::excel::excel_report::{AttendanceBook, ExcelExporter};
use parts::excel::traits::AttendanceApi;
use parts::excel::report_manager::ReportManager;

fn april_days(year: i32) -> Vec<NaiveDate> {
    (1..=30).map(|d| NaiveDate::from_ymd_opt(year, 4, d).unwrap()).collect()
}

#[tokio::main]
async fn main() {

    let header = ReportHeader {
        group: "23ВЕБ-1".to_string(),
        curator: "Бокарёва С.Ф".to_string(),
        starosta: "Шестаков А.Д".to_string(),
        month_label: "Август".to_string(),
        academic_year: "2025-2026".to_string(),
    };

    let days = (1..=30)
        .map(|d| NaiveDate::from_ymd_opt(2025, 9, d).unwrap())
        .collect::<Vec<_>>();

    let mut manager = ReportManager::open_or_create("23ВЕБ-1 август", header, days)
        .expect("Не удалось открыть или создать отчет");

    manager.add_student(Student {
        name: "Петров П.П.".to_string(),
        hours_by_date: Default::default(),
        excused_by_date: Default::default(),
    });

    manager.save().expect("Не удалось сохранить отчет"); // Перезаписать Excel

    println!("Готово: файл «посещаемость.xlsx» создан.");


    let current_dir = env::current_dir();
    println!("Current working directory: {:?}", current_dir);

    let file_name = "Database.db";


    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_name); // Пытаемся открыть/создать файл

    match result {
        Ok(_) => {

            println!("Файл '{}' успешно создан.", file_name);
        }
        Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {

            println!("Файл '{}' уже существует. Создание пропущено.", file_name);
        }
        Err(e) => {

            eprintln!("Не удалось создать файл '{}': {}", file_name, e);
        }
    }


    // Инициализируем БД пул один раз
    parts::db::db_pool::init_db()
        .await
        .expect("Не удалось инициализировать базу данных");

    cache::init_memcached_pool(MEMCACHED_PRODUCTION_ADDRESS, 30)
        .await
        .expect("Не удалось инициализировать memcached пул");

    let cache = cache::MemcachedCache::new();


    // Задача записи начальных значений в memcached
    for group in GROUPS_VEC.iter() {
        let key = format!("{}_weight", group);
        let value = "0".to_string();

        cache.set(&key, &value.as_bytes(), 0)
            .await
            .expect("Memcached недоступен");
    }

    async fn run_with_restart<F>(mut task_fn: F, task_name: &str)
    where
        F: FnMut() -> task::JoinHandle<()> + Send + 'static,
    {
        loop {
            println!("Запуск задачи: {}", task_name);
            tokio::time::sleep(Duration::from_secs(15)).await;
            let task = task_fn();
            match task.await {
                Ok(_) => {
                    println!("Задача {} завершена", task_name);
                    break;
                }
                Err(e) => {
                    println!("Ошибка в задаче {}: {:?}", task_name, e);
                    println!("Перезапуск задачи {}", task_name);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    // Остальные задачи
    let ux_task = tokio::spawn(run_with_restart(
        || task::spawn(async {
            parts::ux::start_ux().await;
        }),
        "start_ux",
    ));

    let replace_task = tokio::spawn(run_with_restart(
        || task::spawn(async {
            parts::cycle::cycle_work_replace().await;
        }),
        "cycle_work_replace",
    ));

    // Ожидаем завершения задач
    let _ = replace_task.await.expect("Ошибка cycle_work_replace");
    let _ = ux_task.await.expect("Ошибка start_ux");

    println!("Все задачи завершены");
}
