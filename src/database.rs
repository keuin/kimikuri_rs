use std::str::FromStr;

use futures::TryStreamExt;
use log::{debug, error, info, warn};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::user::User;

pub type DbPool = sqlx::sqlite::SqlitePool;

const SQLITE_THREAD_POOL_SIZE: u32 = 16;

pub async fn open(file_path: &str) -> Result<DbPool, sqlx::Error> {
    let opt =
        SqliteConnectOptions::from_str(format!("sqlite://{}", file_path).as_str())?
            .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(SQLITE_THREAD_POOL_SIZE)
        .connect_with(opt).await?;

    // create table
    sqlx::query(r#"CREATE TABLE IF NOT EXISTS "user" (
                "id"	INTEGER NOT NULL CHECK(id >= 0) UNIQUE,
                "name"	TEXT,
                "token"	TEXT NOT NULL UNIQUE,
                "chat_id"	INTEGER NOT NULL CHECK(chat_id >= 0) UNIQUE,
                PRIMARY KEY("id","token","chat_id")
        )"#).execute(&pool).await?;

    Ok(pool)
}

pub async fn get_user(db: &DbPool, sql: &str, param: &str) -> Result<Option<User>, sqlx::Error> {
    let mut rows =
        sqlx::query(sql)
            .bind(param).fetch(db);
    let result = rows.try_next().await?;
    match result {
        None => Ok(None),
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let chat_id: i64 = row.try_get("chat_id")?;
            Ok(Some(User {
                id: id as u64,
                name: row.try_get("name")?,
                token: row.try_get("token")?,
                chat_id: chat_id as u64,
            }))
        }
    }
}

pub async fn get_user_by_token(db: &DbPool, token: &str) -> Result<Option<User>, sqlx::Error> {
    get_user(db, "SELECT id, name, token, chat_id FROM user WHERE token=$1 LIMIT 1",
             token).await
}

pub async fn get_user_by_chat_id(db: &DbPool, chat_id: u64) -> Result<Option<User>, sqlx::Error> {
    get_user(db, "SELECT id, name, token, chat_id FROM user WHERE chat_id=$1 LIMIT 1",
             chat_id.to_string().as_str()).await
}

pub async fn create_user(db: &DbPool, user: User) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO user (id, name, token, chat_id) VALUES ($1,$2,$3,$4)")
        .bind(user.id.to_string())
        .bind(user.name)
        .bind(user.token)
        .bind(user.chat_id.to_string())
        .execute(db).await?;
    Ok(())
}