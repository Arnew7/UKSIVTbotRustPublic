use anyhow::Result;

#[async_trait::async_trait]
pub trait CacheInterface: Send + Sync {
    async fn set(&self, key: &str, value: &[u8], expiration_seconds: usize) -> Result<()>;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub struct MemcachedCache;

impl MemcachedCache {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl CacheInterface for MemcachedCache {
    async fn set(&self, key: &str, value: &[u8], expiration_seconds: usize) -> Result<()> {
        let pool = super::memcached_client::get_memcached_pool();
        let mut client = pool.get().await.unwrap();
        client.set(key, value, expiration_seconds as u32)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let pool = super::memcached_client::get_memcached_pool();
        let mut client = pool.get().await.unwrap();
        let res: Option<Vec<u8>> = client.get(key)?;
        Ok(res)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let pool = super::memcached_client::get_memcached_pool();
        let mut client = pool.get().await.unwrap();
        client.delete(key)?;
        Ok(())
    }
}
