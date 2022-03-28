use std::{io, path};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{SocketAddr, ToSocketAddrs};
use std::str::FromStr;

use teloxide::prelude2::*;
use tracing::{debug, info, Level};
use tracing::instrument;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
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

fn with_object<T: Clone + Send>(obj: T)
                                -> impl Filter<Extract=(T, ), Error=Infallible> + Clone {
    warp::any().map(move || obj.clone())
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
                      config.log_level, config::DEFAULT_LOG_LEVEL);
            Level::from_str(config::DEFAULT_LOG_LEVEL).unwrap()
        }
    };
    eprintln!("Configuration is loaded. Set log level to {:?}.", log_level);
    let _guard: WorkerGuard;
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(log_level)
        .with_writer(io::stderr) // log to stderr
        .finish();
    if !config.log_file.is_empty() {
        let log_file_path = path::Path::new(&config.log_file);
        let parent = log_file_path.parent()
            .expect("Invalid log_file: Cannot extract parent.");
        let filename = log_file_path.file_name()
            .expect("Invalid log_file: Cannot extract file name.");
        let (nb_file_appender, guard) = tracing_appender::non_blocking(
            tracing_appender::rolling::never(parent, filename));
        _guard = guard;
        let subscriber = subscriber.with(
            fmt::Layer::default()
                .with_writer(nb_file_appender) // log to file
                .with_ansi(false) // remove color control characters from log file
        );
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set default subscriber");
    } else {
        // log file is not specified, do not write logs to file
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set default subscriber");
    }

    let db = config.db_file.as_str();
    info!(db, "Opening database...");
    let db: DbPool = database::open(db, config.sqlite_thread_pool_size)
        .await.expect(&*format!("cannot open database {}", db));

    info!("Spawning bot coroutine...");
    let bot = Bot::new(config.bot_token);
    tokio::spawn(bot::repl(bot.clone(), db.clone()));

    info!("Initializing HTTP routes...");
    let route_post = warp::post()
        .and(warp::body::content_length_limit(config.max_body_size))
        .and(warp::body::json())
        .and(with_object(db.clone()))
        .and(with_object(bot.clone()))
        .and_then(web::handler);
    let route_get = warp::get()
        .and(warp::query::<HashMap<String, String>>())
        .and(with_object(db.clone()))
        .and(with_object(bot.clone()))
        .and_then(web::get_handler);
    let routes = warp::path("message")
        .and(route_post).or(route_get);

    info!("Starting HTTP server...");
    let endpoint: SocketAddr = config.listen.parse()
        .unwrap_or_else(|_| config.listen
            .to_socket_addrs()
            .expect("Cannot resolve endpoint.")
            .next()
            .expect("Cannot resolve endpoint."));
    info!("Start listening on {}", endpoint);
    tokio::spawn(warp::serve(routes).run(endpoint));

    debug!("Waiting for Ctrl-C in main coroutine...");
    tokio::signal::ctrl_c().await.unwrap();

    // gracefully shutdown the database connection
    info!("Closing database...");
    db.close().await;
}
