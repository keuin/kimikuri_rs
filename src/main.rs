use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

use teloxide::prelude2::*;
use tracing::{debug, info, Level};
use tracing::instrument;
use tracing_subscriber::FmtSubscriber;
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
const DEFAULT_LOG_LEVEL: Level = Level::DEBUG;

fn with_db(db_pool: DbPool) -> impl Filter<Extract=(DbPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

// TODO replace with generic
fn with_bot(bot: Bot) -> impl Filter<Extract=(Bot, ), Error=Infallible> + Clone {
    warp::any().map(move || bot.clone())
}

#[instrument]
#[tokio::main]
async fn main() {
    eprintln!("Loading configuration file {}...", CONFIG_FILE_NAME);
    // TODO make some fields optional
    let config = Config::from_file(CONFIG_FILE_NAME);

    // configure logger
    let log_level = match Level::from_str(&*config.log_level) {
        Ok(l) => l,
        Err(_) => {
            eprintln!("Invalid log level: {}. Use {:?} instead.",
                      config.log_level, DEFAULT_LOG_LEVEL);
            DEFAULT_LOG_LEVEL
        }
    };
    eprintln!("Configuration is loaded. Set log level to {:?}.", log_level);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");

    let db = config.db_file.as_str();
    info!(db, "Opening database...");
    let db: DbPool = database::open(db)
        .await.expect(&*format!("cannot open database {}", db));

    info!("Spawning bot coroutine...");
    let bot = Bot::new(config.bot_token);
    tokio::spawn(bot::repl(bot.clone(), db.clone()));

    info!("Initializing HTTP routes...");
    let route_post = warp::post()
        .and(warp::body::content_length_limit(MAX_BODY_LENGTH))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and(with_bot(bot.clone()))
        .and_then(web::handler);
    let route_get = warp::get()
        .and(warp::query::<HashMap<String, String>>())
        .and(with_db(db.clone()))
        .and(with_bot(bot.clone()))
        .and_then(web::get_handler);
    let routes = warp::path("message")
        .and(route_post).or(route_get);

    info!("Starting HTTP server...");
    let endpoint: SocketAddr = config.listen.parse()
        .expect("Cannot parse `listen` as endpoint.");
    info!("Start listening on {}", endpoint);
    tokio::spawn(warp::serve(routes).run(endpoint));

    debug!("Waiting for Ctrl-C in main coroutine...");
    tokio::signal::ctrl_c().await.unwrap();

    // gracefully shutdown the database connection
    info!("Closing database...");
    db.close().await;
}
