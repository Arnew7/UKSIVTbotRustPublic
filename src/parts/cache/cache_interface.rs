// anyhow::Result — это тип результата из библиотеки anyhow,
// который удобно использовать для возврата ошибок любого типа.
use anyhow::Result;

// async_trait — это макрос, который позволяет объявлять async-функции в трейтах.
// Без него Rust не даёт писать async fn в trait из-за ограничений языка.
#[async_trait::async_trait]
pub trait CacheInterface: Send + Sync {
    // Метод для записи значения в кэш.
    // key — ключ (строка), value — данные (байты), expiration_seconds — время жизни в секундах.
    async fn set(&self, key: &str, value: &[u8], expiration_seconds: usize) -> Result<()>;

    // Метод для получения значения по ключу.
    // Возвращает Option<Vec<u8>> — либо данные, либо None, если ключа нет.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    // Метод для удаления значения из кэша.
    async fn delete(&self, key: &str) -> Result<()>;
}

// Конкретная реализация кэша через Memcached.
pub struct MemcachedCache;

impl MemcachedCache {
    // Конструктор, пока без параметров.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl CacheInterface for MemcachedCache {
    // Запись в Memcached
    async fn set(&self, key: &str, value: &[u8], expiration_seconds: usize) -> Result<()> {
        // Получаем пул подключений к Memcached.
        let pool = super::memcached_client::get_memcached_pool();

        // Берём одно соединение из пула (await, так как пул асинхронный).
        // unwrap() здесь рискован — если соединение не удалось взять, бот упадёт.
        let mut client = pool.get().await.unwrap();

        // Сохраняем данные по ключу с указанным временем жизни.
        client.set(key, value, expiration_seconds as u32)?;

        Ok(())
    }

    // Чтение из Memcached
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let pool = super::memcached_client::get_memcached_pool();
        let mut client = pool.get().await.unwrap();

        // Получаем данные. Если ключа нет — вернётся None.
        let res: Option<Vec<u8>> = client.get(key)?;

        Ok(res)
    }

    // Удаление ключа
    async fn delete(&self, key: &str) -> Result<()> {
        let pool = super::memcached_client::get_memcached_pool();
        let mut client = pool.get().await.unwrap();

        // Удаляем ключ. Если ключа нет — обычно Memcached не считает это ошибкой.
        client.delete(key)?;

        Ok(())
    }
}
