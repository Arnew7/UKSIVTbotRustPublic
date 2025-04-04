use tokio;
use tokio::task;
use tracing_subscriber::{fmt, EnvFilter};
use crate::parts::MyError;

mod parts;
mod Secret;


#[tokio::main]
async fn main() {
    // Инициализируем tracing только один раз в начале программы
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())

        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");

    tracing::info!("Application started");
    let db_task = task::spawn(parts::create_data_base::create_database());
    // Запускаем каждую часть в отдельной задаче
    let ux_task = task::spawn(parts::ux::start_ux());
    let replace_task = task::spawn(parts::Cycle::cycle_work_replace());


    // Добавляем .await для ожидания завершения всех задач, иначе main завершится до их завершения
    ux_task.await.expect("Ошибка при выполнении UX");
    let _ = replace_task.await.expect("Ошибка при выполнении replacements_main");
    let _ = db_task.await.expect("Ошибка при выполнении create_database");

    tracing::info!("All tasks finished");

    println!("Все задачи завершены");
}