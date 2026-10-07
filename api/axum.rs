mod entity;
mod app_state;
mod project_routes;
mod api;
mod team_routes;
mod support_routes;

use std::sync::Arc;
use axum::response::{IntoResponse};
use axum::Router;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use sea_orm::{ActiveModelTrait, ColIdx, EntityTrait, Iden, ModelTrait};
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::Error;
use crate::app_state::AppState;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use crate::project_routes::project_routes;
use crate::support_routes::support_routes;
use crate::team_routes::team_routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().expect(".env file not found");

    let state = Arc::new(AppState::new().await?);

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let router = Router::new()
        .merge(project_routes())
        .merge(team_routes())
        .merge(support_routes())
        .layer(cors)
        .with_state(state);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}
