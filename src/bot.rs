use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

use teloxide::{prelude2::*, types::MessageKind, utils::command::BotCommand};

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

async fn answer(bot: AutoSend<Bot>, message: Message, command: Command, db: Arc<DbPool>)
                -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions()).await?;
        }
        Command::Start => {
            if let MessageKind::Common(msg) = message.kind {
                if msg.from.is_none() {
                    return Ok(()); // ignore messages from channel
                }
                let from = msg.from.unwrap();
                let db = db.deref();
                let chat_id = message.chat.id;
                match database::create_user(db, User {
                    id: from.id as u64,
                    name: from.username.unwrap_or_else(|| String::from("")),
                    token: token::generate(),
                    chat_id: chat_id as u64,
                }).await {
                    Ok(_) => {}
                    Err(why) => println!("cannot create user: {:?}", why),
                }
                bot.send_message(
                    chat_id,
                    match database::get_user_by_chat_id(db, chat_id as u64).await? {
                        Some(user) =>
                            format!("Your token is `{}`. Treat it as a secret!", user.token),
                        _ =>
                            String::from("Error: cannot fetch token.")
                    },
                ).await?;
            }
        }
    };
    Ok(())
}

pub async fn repl(bot: Bot, db: Arc<database::DbPool>) {
    teloxide::repls2::commands_repl(
        bot.auto_send(),
        move |bot, msg, cmd|
            answer(bot, msg, cmd, db.clone()), Command::ty(),
    ).await;
}
