// src/parts/MyError.rs
use std::error::Error as StdError;
use std::string::FromUtf8Error;
use memcache::Error as MemcacheError;
use rusqlite::Error as RusqliteError;

#[derive(Debug)]
pub enum MyError {
    MemcachedError(String),
    Utf8Error(FromUtf8Error),
    NotFoundError(String),
    GenericError(String),
    Other(Box<dyn std::error::Error>),
    Rusqlite(RusqliteError),
    Addbook(AddbookError),
}


impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::MemcachedError(e) => write!(f, "Memcached error: {}", e),
            MyError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
            MyError::NotFoundError(e) => write!(f, "Not found error: {}", e),
            MyError::GenericError(e) => write!(f, "Generic error: {}", e),
            MyError::Rusqlite(e) => write!(f, "Rusqlite error: {}", e), // Corrected typo
            MyError::Other(e) => write!(f, "Other error: {}", e),
            MyError::Addbook(e) => write!(f, "Addbook error: {}", e), // Corrected typo and message
        }
    }
}

impl StdError for MyError {}


impl From<MemcacheError> for MyError {
    fn from(err: MemcacheError) -> Self {
        MyError::MemcachedError(err.to_string())
    }
}
impl From<Box<dyn std::error::Error>> for MyError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        MyError::Other(e)
    }
}

// Remove this From implementation as AddbookError does not implement Error
// impl From<AddbookError> for MyError {
//     fn from(e: AddbookError) -> Self {
//         MyError::Addbook(e)
//     }
// }

impl From<FromUtf8Error> for MyError {
    fn from(err: FromUtf8Error) -> Self {
        MyError::Utf8Error(err)
    }
}

impl From<String> for MyError {
    fn from(err: String) -> Self {
        MyError::NotFoundError(err)
    }
}

impl From<RusqliteError> for MyError {
    fn from(err: RusqliteError) -> Self {
        MyError::Rusqlite(err)
    }
}


unsafe impl Send for MyError {}
unsafe impl Sync for MyError {}

#[derive(Debug)]
pub enum AddbookError {
    ValidationError(String),
    DatabaseError(String),
}

impl std::fmt::Display for AddbookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddbookError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AddbookError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl StdError for AddbookError {}