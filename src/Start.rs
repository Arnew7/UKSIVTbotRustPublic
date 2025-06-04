mod parts;

mod Secret;
use std::time::Duration;
use teloxide::Bot;
use tokio::{select, task, time};
use tracing::subscriber::set_global_default;
use futures::TryFutureExt;
use crate::parts::cache;
use crate::parts::cache::CacheInterface;
use crate::Secret::{MEMCACHED_PRODUCTION_ADDRESS, GROUPS_VEC};

#[tokio::main]
async fn main() {
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
