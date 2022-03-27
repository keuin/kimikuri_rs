use std::fmt;
use std::fmt::Formatter;

use serde_derive::{Deserialize, Serialize};
use teloxide::{prelude2::*};
use tracing::{debug, error, info, warn};
use warp::{Rejection, Reply};

use crate::{Bot, database, DbPool};

#[derive(Deserialize, Serialize)]
pub struct SendMessage {
    token: String,
    message: String,
}

impl fmt::Display for SendMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SendMessage {{ token={}, message={} }}", self.token, self.message)
    }
}

#[derive(Deserialize, Serialize)]
pub struct SendMessageResponse {
    success: bool,
    message: String,
}

impl fmt::Display for SendMessageResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SendMessageResponse {{ success={}, message={} }}", self.success, self.message)
    }
}

pub async fn handler(req: SendMessage, db: DbPool, bot: Bot)
                     -> std::result::Result<impl Reply, Rejection> {
    info!("Income API request: {}", req);
    let user =
        database::get_user_by_token(&db, req.token.as_str()).await;
    let response = match user {
        Ok(u) => match u {
            Some(user) => {
                info!("Send message to user {}.", user);
                let bot = bot.auto_send();
                match bot.send_message(user.chat_id as i64, req.message).await {
                    Ok(_) => SendMessageResponse {
                        success: true,
                        message: String::new(),
                    },
                    Err(why) => {
                        error!("Failed to send message to telegram: {:?}", why);
                        SendMessageResponse {
                            success: false,
                            message: String::from("Failed to send message to telegram."),
                        }
                    }
                }
            }
            None => {
                warn!("Invalid token: {}.", req);
                SendMessageResponse {
                    success: false,
                    message: String::from("Invalid token."),
                }
            }
        },
        Err(err) => {
            error!("Error when querying the database: {:?}.", err);
            SendMessageResponse {
                success: false,
                message: String::from("Invalid parameter."),
            }
        }
    };
    debug!("Response: {}", response);
    Ok(warp::reply::json(&response))
}
