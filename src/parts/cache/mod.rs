pub mod memcached_client;
pub mod cache_interface;

pub use memcached_client::{init_memcached_pool};
pub use cache_interface::{CacheInterface, MemcachedCache};
