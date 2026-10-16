use std::env;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use teloxide::{prelude::*, utils::command::BotCommands};
use crate::entity::public::prelude::Ticket;
use crate::entity::public::ticket;
use crate::entity::public::ticket::Model;

#[derive(BotCommands, Clone)]
#[command(description = "Available commands: ")]
pub enum Command {
    #[command(description = "display the commands", rename="help")]
    Help,
    #[command(description = "resolve the ticket", rename="resolve")]
    Resolve(i32),
    #[command(description = "shows unresolved tickets", rename = "unresolved")]
    Unresolved
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command, db: DatabaseConnection) -> ResponseResult<()> {
    if !msg.chat.id.eq(&ChatId(env::var("ADMIN_ID").unwrap().parse().unwrap())) {
        bot.send_message(msg.chat.id, "You don't have enough permissions").await?;

        return Ok(())
    }

    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, "Help is here").await?
        },
        Command::Resolve(ticket_id) => {
            let ticket = Ticket::find_by_id(ticket_id)
                .one(&db)
                .await;

            match ticket {
                Ok(ticket) => {
                    match ticket {
                        None => {
                            bot.send_message(msg.chat.id, "Ticket not found").await?
                        }
                        Some(ticket) => {
                            if ticket.resolved {
                                bot.send_message(msg.chat.id, "The ticket is already resolved").await?
                            } else {
                                let mut ticket: ticket::ActiveModel = ticket.into();

                                ticket.resolved = Set(true);

                                match ticket.update(&db).await {
                                    Ok(_) => {
                                        bot.send_message(msg.chat.id, "The ticket has been resolved").await?
                                    }
                                    Err(_) => {
                                        bot.send_message(msg.chat.id, "An error occurred").await?
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    bot.send_message(msg.chat.id, "Database lookup failed").await?
                }
            }
        },
        Command::Unresolved => {
            let mut message: String = "Unresolved tickets\n".into();

            let tickets = Ticket::find()
                .filter(ticket::Column::Resolved.eq(false))
                .all(&db)
                .await;



            match tickets {
                Ok(tickets) => {
                    if tickets.len() == 0 {
                        message = "All tickets have been resolved".into();
                    } else {
                        for ticket in tickets {
                            message.push_str(&format!("Ticket Nº{}\n\
                            - Problem: {}\n\n", ticket.id, ticket.message));
                        }
                    }
                }
                Err(_) => {
                    message = "Database lookup failed".into();
                }
            }

            bot.send_message(msg.chat.id, message).await?
        },
    };

    Ok(())
}

pub async fn send_report(bot: &Bot, chat_id: ChatId, error: String) {
    let _ = bot.send_message(chat_id, format!("En error has occurred:\n{}", error)).await;
}