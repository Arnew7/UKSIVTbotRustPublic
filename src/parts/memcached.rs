
use memcache::Client;

use crate::Secret::{MEMCACHED_PRODUCTION_ADDRESS,MEMCACHED_TEST_ADDRESS};
use crate::MyError::MyError;

use std::rc::Rc;

// ... (определение MyError как показано выше - убедитесь, что MyError реализует Send + Sync)

pub async fn get_from_memcached(group: String) -> Result<String, MyError> {
    // 1. Создание подключения к Memcached
    let client_result = Client::connect(MEMCACHED_TEST_ADDRESS);

    let memcached_client = match client_result {
        Ok(client) => Rc::new(client),
        Err(e) => {
            println!("Failed to connect to memcached: {}", e);
            return Err(MyError::Other(Box::new(e)));
        }
    };

    let key = group;

    // 2. Получаем значение по ключу
    let retrieved_bytes_result = memcached_client.get(&key);

    match retrieved_bytes_result {
        Ok(Some(bytes)) => {
            match String::from_utf8(bytes) {
                Ok(retrieved_value) => Ok(retrieved_value),
                Err(e) => {
                    println!("Error converting bytes to string: {}", e);
                    Err(MyError::Utf8Error(e))
                }
            }
        }
        Ok(None) => {
            println!("Key not found in Memcached.");
            Err(MyError::NotFoundError("Key not found in Memcached".to_string()))
        }
        Err(e) => {
            // Явное преобразование ошибки типа memcache::Error в строку
            println!("Error getting value from Memcached: {}", e);
            Err(MyError::MemcachedError(e.to_string()))
        }
    }
}

pub async fn write_on_memcached(text: String, group: String) -> Result<(), MyError> {
    // 1. Создание подключения к Memcached
    let client_result = Client::connect(MEMCACHED_TEST_ADDRESS);
    //для разработки memcache://127.0.0.1:11211
    //для прода memcache://memcached:11211
    let memcached_client = match client_result {
        Ok(client) => Rc::new(client), // Оборачиваем в Rc
        Err(e) => {
            println!("Failed to connect to memcached: {}", e);
            return Err(MyError::Other(Box::new(e)));
        }
    };

    let key = group;
    let value = text;

    // 4. Запись String в Memcached
    match memcached_client.set(key.as_str(), value.as_bytes(), 0) {
        Ok(_) => Ok(()),
        Err(e) => {

            println!("Failed to write to Memcached: {}", e);
            Err(MyError::MemcachedError(e.to_string()))
        }
    }
}
