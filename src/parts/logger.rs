use std::time::{Duration};
use teloxide::Bot;
use crate::parts::send_to_user::send_to_owner;
use crate::Secret::CHAT_ID_OWNER;
pub async fn return_to_owner(duration: Duration) {
    let chat_id_owner = CHAT_ID_OWNER;
    let duration_string = format!("{:?}", duration);
    send_to_owner(duration_string, chat_id_owner).await.unwrap()
}