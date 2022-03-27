use std::convert::Infallible;
use std::sync::Arc;

use log::{debug, error, info, warn};
use teloxide::prelude2::*;
use warp::Filter;

use config::Config;

use crate::database::DbPool;

mod database;
mod user;
mod web;
mod bot;
mod token;
mod config;

const CONFIG_FILE_NAME: &str = "kimikuri.json";
const MAX_BODY_LENGTH: u64 = 1024 * 16;

fn with_db(db_pool: DbPool) -> impl Filter<Extract=(DbPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

// TODO replace with generic
fn with_bot(bot: Bot) -> impl Filter<Extract=(Bot, ), Error=Infallible> + Clone {
    warp::any().map(move || bot.clone())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    debug!("Loading bot config.");
    let config = Config::from_file(CONFIG_FILE_NAME);
    info!("Starting bot.");
    let bot = Bot::new(config.bot_token);

    let db = config.db_file.as_str();
    println!("Database: {}", db);
    let db: DbPool = database::open(db)
        .await.expect(&*format!("cannot open database {}", db));

    let send_message = warp::path("message")
        .and(warp::post())
        .and(warp::body::content_length_limit(MAX_BODY_LENGTH))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and(with_bot(bot.clone()))
        .and_then(web::handler);

    tokio::spawn(bot::repl(bot, Arc::new(db)));

    warp::serve(send_message).run(([127, 0, 0, 1], 3030)).await;
}
