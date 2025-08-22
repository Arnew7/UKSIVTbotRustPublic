use deadpool::managed::{Manager, Pool, RecycleResult}; // Deadpool — асинхронный пул соединений, Manager нужен для создания и управления объектами
use memcache::Client; // Клиент для работы с Memcached
use anyhow::{Result, anyhow}; // anyhow — удобная библиотека для работы с ошибками
use once_cell::sync::OnceCell; // OnceCell — для ленивой инициализации статических переменных

// -----------------------------
// 1. Структура-менеджер соединений
// -----------------------------

/// Менеджер для Memcached, хранит строку подключения.
/// Deadpool будет вызывать методы create() и recycle().
pub struct MemcachedManager {
    connection_str: String, // например: "memcache://127.0.0.1:11211"
}

// -----------------------------
// 2. Реализация Manager для Deadpool
// -----------------------------

#[async_trait::async_trait]
impl Manager for MemcachedManager {
    type Type = Client;         // Тип объекта, которым управляет пул (здесь memcache::Client)
    type Error = anyhow::Error; // Тип ошибок, который мы используем

    /// Создание нового соединения
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        // Подключаемся к Memcached
        let client = Client::connect(&*self.connection_str)?;
        Ok(client)
    }

    /// Проверка или повторная подготовка объекта перед повторным использованием
    async fn recycle(&self, _obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        // Здесь можно реализовать проверку соединения, например, через ping.
        // Сейчас просто возвращаем Ok(), считая, что соединение всегда рабочее.
        Ok(())
    }
}

// -----------------------------
// 3. Тип пула
// -----------------------------

/// Упрощённый алиас для пула соединений с Memcached
pub type MemcachedPool = Pool<MemcachedManager>;

// -----------------------------
// 4. Глобальное хранилище пула
// -----------------------------

/// Глобальная переменная для хранения пула.
/// OnceCell гарантирует, что инициализация произойдёт только один раз.
static MEMCACHED_POOL: OnceCell<MemcachedPool> = OnceCell::new();

// -----------------------------
// 5. Инициализация пула
// -----------------------------

/// Инициализирует пул соединений.
/// Вызывается один раз при старте приложения.
pub async fn init_memcached_pool(connection_str: &str, max_size: usize) -> Result<()> {
    let manager = MemcachedManager {
        connection_str: connection_str.to_string(),
    };

    // Создаём пул с заданным максимальным количеством соединений
    let pool = Pool::builder(manager)
        .max_size(max_size)
        .build()
        .map_err(|e| anyhow!("Ошибка при создании Memcached пула: {e}"))?;

    // Пробуем положить пул в OnceCell
    MEMCACHED_POOL
        .set(pool)
        .map_err(|_| anyhow!("Memcached pool уже инициализирован"))?;

    Ok(())
}

// -----------------------------
// 6. Получение пула
// -----------------------------

/// Возвращает ссылку на глобальный пул.
/// Если инициализация не была выполнена — паника.
pub fn get_memcached_pool() -> &'static MemcachedPool {
    MEMCACHED_POOL.get().expect("Memcached pool не инициализирован")
}
