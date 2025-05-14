use crate::parts::MyError;

mod parts;
mod Secret;



use std::time::Duration;
use futures::TryFutureExt;
use tokio::{select, time, task};
use tracing::subscriber::set_global_default;

use crate::parts::memcached::write_on_memcached;

#[tokio::main]
async fn main() {
    // Инициализация начальной точки сравнения размера замен
    write_on_memcached("Start".to_string(), "Weight".to_string()).await.unwrap();
    println!("Weight инициализирован");

    // Функция для запуска задачи с обработкой ошибок и перезапуском
    async fn run_with_restart<F>(mut task_fn: F, task_name: &str)
    where
        F: FnMut() -> task::JoinHandle<()> + Send + 'static,
    {
        loop {
            println!("Запуск задачи: {}", task_name);
            let task = task_fn();
            match task.await {
                Ok(_) => {
                    println!("Задача {} успешно завершена", task_name);
                    break; // Если задача завершилась успешно, выходим из цикла
                }
                Err(e) => {
                    println!("Ошибка в задаче {}: {:?}", task_name, e);
                    println!("Перезапуск задачи {}", task_name);
                    time::sleep(Duration::from_secs(5)).await; // Задержка перед перезапуском
                }
            }
        }
    }

    // Оборачиваем задачи, которые возвращают rusqlite::Result<()>, чтобы они возвращали ()
    let db_task = tokio::spawn(run_with_restart(
        || task::spawn(async {
            if let Err(err) = parts::create_data_base::create_database().await {
                println!("Ошибка в create_database: {:?}", err);
            }
        }),
        "create_database",
    ));

    let ux_task = tokio::spawn(run_with_restart(
        || task::spawn(async {
            parts::ux::start_ux().await; // Если start_ux возвращает ()
        }),
        "start_ux",
    ));

    let replace_task = tokio::spawn(run_with_restart(
        || task::spawn(async {
            parts::Cycle::cycle_work_replace().await; // Если cycle_work_replace возвращает ()
        }),
        "cycle_work_replace",
    ));

    // Ожидаем завершения всех задач
    let _ = db_task.await.expect("Ошибка при выполнении create_database");
    let _ = ux_task.await.expect("Ошибка при выполнении UX");
    let _ = replace_task.await.expect("Ошибка при выполнении replacements_main");


    println!("Все задачи завершены");
}