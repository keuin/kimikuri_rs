use serde_derive::{Deserialize, Serialize};
use teloxide::{prelude2::*};
use warp::{Rejection, Reply};

use crate::{Bot, database, DbPool};

#[derive(Deserialize, Serialize)]
pub struct SendMessage {
    token: String,
    message: String,
}

#[derive(Deserialize, Serialize)]
pub struct SendMessageResponse {
    success: bool,
    message: String,
}

pub async fn handler(req: SendMessage, db: DbPool, bot: Bot) -> std::result::Result<impl Reply, Rejection> {
    println!("Token: {}, Message: {}", req.token, req.message);
    let user = database::get_user_by_token(&db, req.token.as_str()).await;
    Ok(warp::reply::json(&match user {
        Ok(u) => match u {
            Some(user) => {
                log::info!("User: {} (id={}), message: {}",
                    user.name, user.id, req.message);
                // TODO send message to Telegram
                let bot = bot.auto_send();
                match bot.send_message(user.chat_id as i64, req.message).await {
                    Ok(_) => SendMessageResponse {
                        success: true,
                        message: String::new(),
                    },
                    Err(why) => {
                        println!("Failed to send message to telegram: {:?}", why);
                        SendMessageResponse {
                            success: false,
                            message: String::from("Failed to send message to telegram."),
                        }
                    }
                }
            }
            None => {
                log::warn!("Invalid token {}, message: {}", req.token, req.message);
                SendMessageResponse {
                    success: false,
                    message: String::from("Invalid token."),
                }
            }
        },
        Err(_) => {
            log::error!("Error when querying the database.");
            SendMessageResponse {
                success: false,
                message: String::from("Invalid parameter."),
            }
        }
    }))
}
