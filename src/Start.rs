use crate::parts::MyError;

mod parts;
mod Secret;


use tracing::{info, error};
use tracing_subscriber::{fmt, EnvFilter};
use std::time::Duration;
use futures::TryFutureExt;
use tokio::{select, time, task};
use tracing::subscriber::set_global_default;







#[tokio::main]
async fn main() {
    // Инициализируем tracing только один раз в начале программы
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");

    tracing::info!("Application started");

    // Функция для запуска задачи с обработкой ошибок и перезапуском
    async fn run_with_restart<F>(mut task_fn: F, task_name: &str)
    where
        F: FnMut() -> task::JoinHandle<()> + Send + 'static,
    {
        loop {
            tracing::info!("Запуск задачи: {}", task_name);
            let task = task_fn();
            match task.await {
                Ok(_) => {
                    tracing::info!("Задача {} успешно завершена", task_name);
                    break; // Если задача завершилась успешно, выходим из цикла
                }
                Err(e) => {
                    tracing::error!("Ошибка в задаче {}: {:?}", task_name, e);
                    tracing::warn!("Перезапуск задачи {}", task_name);
                    time::sleep(Duration::from_secs(5)).await; // Задержка перед перезапуском
                }
            }
        }
    }

    // Оборачиваем задачи, которые возвращают rusqlite::Result<()>, чтобы они возвращали ()
    let db_task = tokio::spawn(run_with_restart(
        || task::spawn(async {
            if let Err(err) = parts::create_data_base::create_database().await {
                tracing::error!("Ошибка в create_database: {:?}", err);
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

    tracing::info!("All tasks finished");
    println!("Все задачи завершены");
}