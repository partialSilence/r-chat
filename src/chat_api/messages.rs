use chrono::{DateTime, Utc};

pub struct Message {
    pub id:i64,
    pub text: String,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub created_at: DateTime<Utc>,
    pub is_read: bool
}
impl Message {
    pub fn new(id:i64, text:String, sender_id: i32, receiver_id:i32, created_at:DateTime<Utc>
    , is_read:bool) -> Self {
        Self {
            id, text, sender_id, receiver_id, created_at, is_read
        }
    }
}