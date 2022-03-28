use std::error::Error;

use teloxide::{prelude2::*, types::MessageKind, utils::command::BotCommand};
use tracing::{debug, error, info};

use crate::{database, DbPool, token};
use crate::user::User;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "get your personal token.")]
    Start,
}

async fn answer(bot: AutoSend<Bot>, message: Message, command: Command, db: DbPool)
                -> Result<(), Box<dyn Error + Send + Sync>> {
    debug!("Answering telegram message: {:?}", message);
    match command {
        Command::Help => {
            debug!("Command /help.");
            if let Err(why) =
            bot.send_message(message.chat.id, Command::descriptions()).await {
                error!("Failed to send telegram message: {:?}.", why);
            }
        }
        Command::Start => {
            debug!("Command /start.");
            if let MessageKind::Common(msg) = message.kind {
                if msg.from.is_none() {
                    debug!("Ignore message from channel.");
                    return Ok(()); // ignore messages from channel
                }
                let from = msg.from.unwrap();
                let chat_id = message.chat.id;
                let user = User {
                    id: from.id as u64,
                    name: from.username.unwrap_or_else(|| String::from("")),
                    token: token::generate(),
                    chat_id: chat_id as u64,
                };
                if let Err(why) = database::create_user(&db, &user).await {
                    if format!("{:?}", why).contains("UNIQUE constraint failed") {
                        info!("User exists: {}", user);
                    } else {
                        error!("Failed to create user {}: {:?}. Skip creating.", user, why);
                    }
                } else {
                    info!("Created user {}.", user);
                }
                let message =
                    match database::get_user_by_chat_id(&db, chat_id as u64).await {
                        Ok(u) => match u {
                            Some(user) => format!("Your token is `{}`. Treat it as a secret!", user.token),
                            _ => String::from("Error: cannot fetch token.")
                        },
                        Err(why) => {
                            error!("Cannot get user: {:?}.", why);
                            String::from("Error: cannot fetch token.")
                        }
                    };
                if let Err(why) = bot.send_message(chat_id, message).await {
                    error!("Failed to send telegram message: {:?}.", why);
                }
            }
        }
    };
    Ok(())
}

pub async fn repl(bot: Bot, db: database::DbPool) {
    teloxide::repls2::commands_repl(
        bot.auto_send(),
        move |bot, msg, cmd|
            answer(bot, msg, cmd, db.clone()), Command::ty(),
    ).await;
}
