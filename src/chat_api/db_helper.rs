use crate::chat_api::auth::{CreateUser, User};
use crate::chat_api::routes::Login;
use deadpool_sqlite::{Pool, PoolError};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

pub async fn create_user(user: User, pool: &Pool) -> Result<i64, DbHelperError> {
    let connection = pool.get().await?;
    let result = connection
        .interact(move |conn| {
            let mut stmt = conn
                .prepare("INSERT INTO users (username, name, password_hash) values (?1, ?2, ?3)")?;
            let id = stmt.insert([&user.username, &user.name, &user.get_password_hash()])?;
            Ok::<i64, DbHelperError>(id)
        })
        .await??;
    Ok(result)
}

pub async fn check_user(login: Login, pool: &Pool) -> Result<Option<User>, DbHelperError> {
    let connection = pool.get().await?;
    let Login { username, password } = login;
    let user = connection
        .interact(move |conn| {
            let mut stmt = conn.prepare(
                "Select id, username, name, password_hash from users \
         where username = ?1 limit 1",
            )?;
            let mut rows = stmt.query_map([username], |row| {
                Ok(User::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            })?;
            rows.next().unwrap()
        })
        .await??;
    if bcrypt::verify(password, &user.get_password_hash()).is_ok() {
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

pub async fn initialize_db(pool: &Pool) -> Result<(), DbHelperError> {
    let connection = pool.get().await?;
    connection
        .interact(|conn| {
            conn.execute(
                "Create Table IF NOT EXISTS users (\
        id INTEGER PRIMARY KEY AUTOINCREMENT,\
        username TEXT NOT NULL UNIQUE,\
        name TEXT NOT NULL,\
        password_hash TEXT NOT NULL)",
                (),
            )
        })
        .await??;
    init_users(&pool).await
}

#[derive(Debug)]
pub enum DbHelperError {
    SqliteError(rusqlite::Error),
    DeadpoolError(deadpool_sqlite::InteractError),
    PoolError(deadpool_sqlite::PoolError),
}
impl fmt::Display for DbHelperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DbHelperError::SqliteError(ref err) => write!(f, "SqliteError: {}", err),
            DbHelperError::DeadpoolError(ref err) => write!(f, "DeadpoolError: {}", err),
            DbHelperError::PoolError(ref err) => write!(f, "PoolError: {}", err),
        }
    }
}
impl Error for DbHelperError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            DbHelperError::SqliteError(ref err) => Some(err),
            DbHelperError::DeadpoolError(ref err) => Some(err),
            DbHelperError::PoolError(ref err) => Some(err),
        }
    }
}
impl From<rusqlite::Error> for DbHelperError {
    fn from(value: rusqlite::Error) -> Self {
        DbHelperError::SqliteError(value)
    }
}
impl From<deadpool_sqlite::InteractError> for DbHelperError {
    fn from(value: deadpool_sqlite::InteractError) -> Self {
        DbHelperError::DeadpoolError(value)
    }
}
impl From<deadpool_sqlite::PoolError> for DbHelperError {
    fn from(value: PoolError) -> Self {
        Self::PoolError(value)
    }
}
async fn init_users(pool: &Pool) -> Result<(), DbHelperError> {
    let connection = pool.get().await?;
    let result: i32 = connection
        .interact(|conn| {
            let mut stmt = conn.prepare("Select Count(*) from users")?;
            let mut rows = stmt.query([])?;
            let row = rows.next()?.unwrap();
            row.get(0)
        })
        .await??;
    if result != 0 {
        return Ok(());
    }
    let users: Vec<User> = get_create_users()
        .into_iter()
        .map(|u| User::from(u))
        .collect();
    connection
        .interact(|conn| {
            let mut stmt = conn
                .prepare("INSERT INTO users (username, name, password_hash) values (?1, ?2, ?3)")
                .unwrap();
            for u in users.into_iter() {
                stmt.execute([&u.username, &u.name, &u.get_password_hash()])
                    .unwrap();
            }
        })
        .await?;
    Ok(())
}

fn get_create_users() -> Vec<CreateUser> {
    Vec::from([
        CreateUser {
            username: String::from("test"),
            name: String::from("test"),
            password: String::from("test"),
        },
        CreateUser {
            username: String::from("test1"),
            name: String::from("test"),
            password: String::from("test"),
        },
        CreateUser {
            username: String::from("test2"),
            name: String::from("test"),
            password: String::from("test"),
        },
        CreateUser {
            username: String::from("test3"),
            name: String::from("test"),
            password: String::from("test"),
        },
    ])
}
