use std::str::FromStr;
use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::post;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use crate::api::APIResponse;
use crate::app_state::AppState;
use teloxide::prelude::*;
use crate::bot::send_report;
use crate::entity::auth::prelude::User;
use crate::entity::public::ticket;

#[derive(Deserialize, Debug)]
struct TicketBody {
    user: Uuid,
    message: String,
    telegram: Option<String>
}

#[derive(Deserialize, Debug)]
struct AnonymousTicketBody {
    email: Option<String>,
    message: String,
    telegram: Option<String>
}

#[derive(Serialize)]
struct TicketRes {
    message: String
}

async fn create_support_request(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TicketBody>
) -> APIResponse<TicketRes> {
    let bot = &state.bot;
    let user = User::find_by_id(body.user).one(&state.db).await;

    println!("{body:?}");

    match user {
        Ok(user) => {
            match user {
                None => {
                    APIResponse::Error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "User not found".into()
                    )
                }
                Some(user) => {
                    let ticket = ticket::ActiveModel {
                        id: Default::default(),
                        user: Set(body.user),
                        message: Set(body.message),
                        telegram: Set(body.telegram),
                        created_at: Default::default(),
                        resolved: Default::default(),
                    };

                    let ticket = ticket.insert(&state.db).await;

                    match ticket {
                        Ok(ticket) => {
                            let message = format!("Ticket Nº{}\n\
                            Created by {}\n\
                            Time {}\n\
                            Telegram {}\n\
                            Email {} \n\
                            \n\
                            Message:\n\
                            {}",
                              ticket.id,
                              user.name,
                              ticket.created_at,
                              ticket.telegram.unwrap_or("not provided".into()),
                              user.email,
                              ticket.message);

                            let res = bot
                                .send_message(state.chat_id, message)
                                .await;

                            match res {
                                Ok(_) => {
                                    APIResponse::OK
                                }
                                Err(err) => {
                                    send_report(&state.bot, state.chat_id, err.to_string()).await;
                                    
                                    APIResponse::Error(
                                        Default::default(),
                                        "Message not sent".into()
                                    )
                                }
                            }
                        }
                        Err(err) => {
                            send_report(&state.bot, state.chat_id, err.to_string()).await;

                            APIResponse::Error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Operation failed".into()
                            )
                        }
                    }
                }
            }
        }
        Err(err) => {
            send_report(&state.bot, state.chat_id, err.to_string()).await;
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn create_anonymous_support_request(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AnonymousTicketBody>
) -> APIResponse<TicketRes> {
    let bot = &state.bot;

    let ticket = ticket::ActiveModel {
        id: Default::default(),
        user: Set(Uuid::from_str("161ad7ec-e6c5-4370-a9f9-74f7547d89d8").unwrap()),
        message: Set(body.message),
        telegram: Set(body.telegram),
        created_at: Default::default(),
        resolved: Default::default(),
    };

    let ticket = ticket.insert(&state.db).await;

    match ticket {
        Ok(ticket) => {
            let message = format!("Ticket Nº{}\n\
            Created by {}\n\
            Time {}\n\
            Telegram {}\n\
            Email {} \n\
            \n\
            Message:\n\
            {}",
              ticket.id,
              "Anonymous",
              ticket.created_at,
              ticket.telegram.unwrap_or("not provided".into()),
              body.email.unwrap_or("not provided".into()),
              ticket.message);

            let res = bot
                .send_message(state.chat_id, message)
                .await;

                match res {
                    Ok(_) => {
                        APIResponse::OK
                    }
                    Err(err) => {
                        send_report(&state.bot, state.chat_id, err.to_string()).await;
                        
                        APIResponse::Error(
                            Default::default(),
                            "Message not sent".into()
                        )
                    }
                }
        }
        Err(err) => {
            send_report(&state.bot, state.chat_id, err.to_string()).await;

            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Operation failed".into()
            )
        }
    }
}

pub fn support_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/support", post(create_support_request))
        .route("/support/anonymous", post(create_anonymous_support_request))
}