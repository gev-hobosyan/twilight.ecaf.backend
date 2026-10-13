mod entity;
mod app_state;
mod project_routes;
mod api;
mod team_routes;
mod support_routes;
mod bot;
mod admin_routes;

use std::env;
use std::sync::Arc;
use axum::body::Bytes;
use axum::response::{IntoResponse};
use axum::Router;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::routing::get;
use sea_orm::{ActiveModelTrait, ColIdx, EntityTrait, Iden, ModelTrait};
use vercel_runtime::Error;
use crate::app_state::AppState;
use dotenvy::dotenv;
use genpdf::{fonts, Document, SimplePageDecorator, elements, Alignment, Element, style};
use genpdf::fonts::{Font, FontData, FontFamily};
use hyper::{header, HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use teloxide::prelude::Requester;
use teloxide::repls::CommandReplExt;
use tower_http::cors::CorsLayer;
use crate::admin_routes::admin_routes;
use crate::bot::{answer, Command};
use crate::project_routes::project_routes;
use crate::support_routes::support_routes;
use crate::team_routes::team_routes;

async fn test_pdf() -> Result<impl IntoResponse, StatusCode> {
    let font = fonts::from_files("/Users/gevorghobosyan/twilight.ecaf.backend/fonts",
    "Audiowide", Some(fonts::Builtin::Helvetica)).unwrap();

    let mut document = Document::new(font);
    document.set_title("Test");
    document.set_minimal_conformance();
    document.set_line_spacing(1.25);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    document.set_page_decorator(decorator);

    document.push(
        elements::Paragraph::new("Test document")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(20))
    );
    document.push(elements::Break::new(1.5));
    document.push(elements::Paragraph::new(
        "This document demonstrates how the genpdf crate generates PDF documents. Currently, \
         genpdf supports these elements:",
    ));

    let mut pdf_bytes: Vec<u8> = Vec::new();
    document.render(&mut pdf_bytes).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "inline; filename=\"document.pdf\"".parse().unwrap()
    );

    Ok((headers, Bytes::from(pdf_bytes)))
}

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
        }
    );

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let router = Router::new()
        .merge(project_routes())
        .merge(team_routes())
        .merge(support_routes())
        .merge(admin_routes())
        .route("/pdf", get(test_pdf))
        .layer(cors)
        .with_state(state);

    let port = env::var("PORT").unwrap_or("3000".into());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await?;

    axum::serve(listener, router).await?;

    Ok(())
}
