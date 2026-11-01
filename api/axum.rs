mod entity;
mod app_state;
mod project_routes;
mod api;
mod team_routes;
mod support_routes;
mod bot;
mod admin_routes;
mod pdf_utils;
mod user_routes;
mod progress_routes;
mod vote;
mod vote_event;

use std::env;
use std::sync::Arc;
use axum::response::{IntoResponse};
use axum::Router;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use dotenvy::dotenv;
use sea_orm::{ActiveModelTrait, ColIdx, EntityTrait, Iden, ModelTrait};
use vercel_runtime::Error;
use crate::app_state::AppState;
use serde::{Deserialize, Serialize};
use teloxide::repls::CommandReplExt;
use tower_http::cors::{AllowOrigin, CorsLayer};
use crate::admin_routes::admin_routes;
use crate::bot::{answer, Command};
use crate::progress_routes::progress_routes;
use crate::project_routes::project_routes;
use crate::support_routes::support_routes;
use crate::team_routes::team_routes;
use crate::user_routes::user_routes;
use crate::vote::vote_routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().expect(".env file not found");

    let state = Arc::new(AppState::new().await?);

    let bot = state.bot.clone();
    let db = state.db.clone();

    tokio::spawn(async move {
        Command::repl(
            bot,
            move |bot, msg, cmd| answer(bot, msg, cmd, db.clone())
        ).await;
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            "http://localhost:3000".parse::<axum::http::HeaderValue>()?,
            "https://twilightecaf.vercel.app".parse::<axum::http::HeaderValue>()?
        ]))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, "x-api-key".parse::<axum::http::HeaderName>()?]);

    let router = Router::new()
        .merge(project_routes())
        .merge(team_routes())
        .merge(support_routes())
        .merge(admin_routes())
        .merge(user_routes())
        .merge(progress_routes())
        .merge(vote_routes())
        .layer(cors)
        .with_state(state);

    let port = env::var("PORT").unwrap_or("3000".into());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await?;

    axum::serve(listener, router).await?;

    Ok(())
}
