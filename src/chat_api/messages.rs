use crate::chat_api::db_helper::DbHelperError;
use chrono::{DateTime, Utc};
use deadpool_sqlite::Pool;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use crate::chat_api::auth::User;

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: i64,
    pub text: String,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}
impl Message {
    pub fn new(
        id: i64,
        text: String,
        sender_id: i32,
        receiver_id: i32,
        created_at: DateTime<Utc>,
        is_read: bool,
    ) -> Self {
        Self {
            id,
            text,
            sender_id,
            receiver_id,
            created_at,
            is_read,
        }
    }
}

pub async fn create_table_messages(pool: &Pool) -> Result<(), DbHelperError> {
    let connection = pool.get().await?;
    connection
        .interact(|conn| {
            let mut stmt = conn.prepare(
                "Create table IF NOT EXISTS messages(
	id INTEGER PRIMARY KEY AUTOINCREMENT,
  text TEXT NOT NULL,
  created_at DATETIME,
  sender_id INT,
  receiver_id INT,
  is_read INT2 DEFAULT 0,
  FOREIGN KEY (sender_id) REFERENCES users(id),
  FOREIGN KEY (receiver_id) REFERENCES users(id)
)",
            )?;
            stmt.execute([])?;
            Ok::<(), DbHelperError>(())
        })
        .await??;
    Ok(())
}


pub async fn create_message(pool: &Pool, mut message:Message) -> Result<Message, DbHelperError> {
    let connection = pool.get().await?;
     connection.interact(move |conn| {
        let mut stmt = conn.prepare("insert into messages (text, created_at, sender_id,\
        receiver_id, is_read) values (?1, ?2, ?3, ?4, ?5)")?;
        let result = stmt.insert((&message.text, &message.created_at,
        message.sender_id, message.receiver_id, message.is_read))?;
        message.id = result;
        Ok::<Message, DbHelperError>(message)
    }).await?
}
pub async fn get_message_by_id(pool: &Pool, id:i64) -> Result<Option<Message>, DbHelperError> {
    let conn = pool.get().await?;
    let result = conn.interact(move |conn| {
        let mut stmt = conn.prepare("Select * from messages \
        where id = ?1")?;
        stmt.query_row([id], |row| {
            Ok(Message::new(row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?
            ))
        }).optional()

    }).await??;
    Ok(result)
}

pub async fn delete_message(pool: &Pool, message_id: i64, user_id:i32 )
    -> Result<(), DbHelperError> {
    let conn = pool.get().await?;
    let result = conn.interact( move |conn| {
        let mut stmt = conn.prepare("delete from messages where id = ?1 and \
        sender_id = ?2")?;
        eprintln!("Try remove message with id {} from user with id {}", message_id, user_id);
        stmt.execute((message_id, user_id))
    }).await??;
    eprintln!("count rows removed: {result}");
    Ok(())
}
pub async fn get_messages_thread(pool: &Pool, (first_user_id, sec_user_id): (i32, i32) )
    -> Result<Vec<Message>, DbHelperError> {
    let conn = pool.get().await?;
    conn.interact(move |conn| {
        let mut stmt = conn.prepare("Select * from messages \
        where (sender_id = ?1 and receiver_id = ?2) or (sender_id = ?2 and receiver_id = ?1)")?;
        let mut rows = stmt.query_map([first_user_id, sec_user_id], |row|{
            Ok(Message::new(
                row.get("id")?,
                row.get("text")?,
                row.get("sender_id")?,
                row.get("receiver_id")?,
                row.get("created_at")?,
                row.get("is_read")?
            ))
        })?;
        Ok::<_, DbHelperError>(rows.flat_map(|e| {
            e
        }).collect::<Vec<Message>>())
    }).await?
}
pub async fn get_message_threads(pool: &Pool, user_id: i32) -> Result<Vec<Message>, DbHelperError> {
    let conn = pool.get().await?;
    conn.interact( move |conn| {
        let mut stmt = conn.prepare("select id, text, sender_id, receiver_id, created_at, is_read
from messages
where sender_id = ?1 or receiver_id = ?1
group by
Case
when sender_id > receiver_id then receiver_id
else sender_id end,
case
when sender_id < receiver_id then receiver_id
else sender_id end
order by created_at")?;
        let result = stmt.query_map([user_id], |row| {
            Ok(Message::new(
                row.get("id")?,
                row.get("text")?,
                row.get("sender_id")?,
                row.get("receiver_id")?,
                row.get("created_at")?,
                row.get("is_read")?
            ))
        })?;
        Ok::<_, DbHelperError>(result.flat_map(|e| {
            e
        }).collect::<Vec<Message>>())
    }).await?
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub text: String,
    pub receiver_id: i32
}

impl From<CreateMessage> for Message {
    fn from(value: CreateMessage) -> Self {
        Self {
            id: 0,
            text:value.text,
            sender_id: 0,
            created_at: Utc::now(),
            receiver_id: value.receiver_id,
            is_read: false
        }
    }
}