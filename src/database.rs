use std::str::FromStr;

use futures::TryStreamExt;
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tracing::debug;

use crate::user::User;

pub type DbPool = sqlx::sqlite::SqlitePool;

pub async fn open(file_path: &str, sqlite_thread_pool_size: u32) -> Result<DbPool, sqlx::Error> {
    let opt =
        SqliteConnectOptions::from_str(format!("sqlite://{}", file_path).as_str())?
            .create_if_missing(true);
    debug!("Opening database pool...");
    let pool = SqlitePoolOptions::new()
        .max_connections(sqlite_thread_pool_size)
        .connect_with(opt).await?;

    // create table
    debug!("Creating user table if not exist...");
    sqlx::query(r#"CREATE TABLE IF NOT EXISTS "user" (
                "id"	INTEGER NOT NULL CHECK(id >= 0) UNIQUE,
                "name"	TEXT,
                "token"	TEXT NOT NULL UNIQUE,
                "chat_id"	INTEGER NOT NULL CHECK(chat_id >= 0) UNIQUE,
                PRIMARY KEY("id","token","chat_id")
        )"#).execute(&pool).await?;

    debug!("Finish opening database.");
    Ok(pool)
}

pub async fn get_user(db: &DbPool, sql: &str, param: &str) -> Result<Option<User>, sqlx::Error> {
    debug!(sql, param, "Database query.");
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
    debug!(token, "Get user by token.");
    get_user(db, "SELECT id, name, token, chat_id FROM user WHERE token=$1 LIMIT 1",
             token).await
}

pub async fn get_user_by_chat_id(db: &DbPool, chat_id: u64) -> Result<Option<User>, sqlx::Error> {
    debug!(chat_id, "Get user by chat_id.");
    get_user(db, "SELECT id, name, token, chat_id FROM user WHERE chat_id=$1 LIMIT 1",
             chat_id.to_string().as_str()).await
}

pub async fn create_user(db: &DbPool, user: &User) -> sqlx::Result<()> {
    debug!("Create user: {}", user);
    sqlx::query("INSERT INTO user (id, name, token, chat_id) VALUES ($1,$2,$3,$4)")
        .bind(user.id.to_string())
        .bind(user.name.clone())
        .bind(user.token.clone())
        .bind(user.chat_id.to_string())
        .execute(db).await?;
    Ok(())
}