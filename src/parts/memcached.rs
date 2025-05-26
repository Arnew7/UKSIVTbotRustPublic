
use memcache::Client;

use crate::Secret::{MEMCACHED_PRODUCTION_ADDRESS,MEMCACHED_TEST_ADDRESS};
use crate::MyError::MyError;

use tokio::task;
use crate::parts::MyError::MyError::MemcachedError;

pub async fn get_from_memcached(group: String) -> Result<String, MyError> {
    let key = group;


    let result = task::spawn_blocking(move || {
        let client = Client::connect(MEMCACHED_PRODUCTION_ADDRESS)
            .map_err(|e| MemcachedError(e.to_string()))?;
        let retrieved_bytes_result = client.get(&key);

        match retrieved_bytes_result {
            Ok(Some(bytes)) => {
            String::from_utf8(bytes)
                .map_err(|e| MyError::Utf8Error(e))
        }
            Ok(None) => {
                println!("Key not found in Memcached.");
                Err(MyError::NotFoundError("Key not found in Memcached".to_string()))
            }
            Err(e) => {
                println!("Error getting value from Memcached: {}", e);
                Err(MyError::MemcachedError(e.to_string()))
            }
        }

    }).await;

    result.unwrap_or_else(|join_error| {
        println!("Blocking task for get_from_memcached failed: {}", join_error);
        Err(MyError::Join(join_error))
    })
}

pub async fn write_on_memcached(text: String, group: String) -> Result<(), MyError> {
    let key = group;
    let value = text;

    let result = task::spawn_blocking(move || {
        let client = Client::connect(MEMCACHED_PRODUCTION_ADDRESS)
            .map_err(|e| MemcachedError(e.to_string()))?;

        client.set(key.as_str(), value.as_bytes(), 0)
            .map_err(|e| MemcachedError(e.to_string()))?;

        Ok(())
    }).await;

    result.unwrap_or_else(|join_error| {
        println!("Blocking task for write_on_memcached failed: {}", join_error);
        Err(MyError::Join(join_error))
    })
}


