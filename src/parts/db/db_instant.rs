use once_cell::sync::Lazy;
use crate::parts::db::interface_db::GlobalDB;

pub static DB: Lazy<GlobalDB> = Lazy::new(|| GlobalDB);
