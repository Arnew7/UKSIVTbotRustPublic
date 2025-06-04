use deadpool::managed::{Manager, Pool, RecycleResult};
use memcache::Client;
use anyhow::{Result, anyhow};
use once_cell::sync::OnceCell;


pub struct MemcachedManager {
    connection_str: String,
}

#[async_trait::async_trait]
impl Manager for MemcachedManager {
    type Type = Client;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let client = Client::connect(&*self.connection_str)?;
        Ok(client)
    }

    async fn recycle(&self, _obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        // Можно добавить проверку, например ping
        Ok(())
    }
}

pub type MemcachedPool = Pool<MemcachedManager>;

static MEMCACHED_POOL: OnceCell<MemcachedPool> = OnceCell::new();

pub async fn init_memcached_pool(connection_str: &str, max_size: usize) -> Result<()> {
    let manager = MemcachedManager {
        connection_str: connection_str.to_string(),
    };

    let pool = Pool::builder(manager)
        .max_size(max_size)
        .build()
        .map_err(|e| anyhow!("Ошибка при создании Memcached пула: {e}"))?; // <- распаковываем здесь

    MEMCACHED_POOL
        .set(pool)
        .map_err(|_| anyhow!("Memcached pool уже инициализирован"))?;

    Ok(())
}

pub fn get_memcached_pool() -> &'static MemcachedPool {
    MEMCACHED_POOL.get().expect("Memcached pool не инициализирован")
}
