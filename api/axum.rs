mod entity;
mod app_state;
mod project_routes;
mod api;
mod team_routes;

use std::env;
use std::sync::Arc;
use axum::response::{IntoResponse};
use axum::{Json, Router};
use sea_orm::{ActiveModelTrait, ColIdx, EntityTrait, Iden, ModelTrait};
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::Error;
use crate::app_state::AppState;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use crate::project_routes::project_routes;
use crate::team_routes::team_routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().expect(".env file not found");

    let db_url = env::var("NEON_DATABASE_URL").unwrap();

    let state = Arc::new(AppState::new(db_url).await);

    let router = Router::new()
        .merge(project_routes())
        .merge(team_routes())
        .with_state(state);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}
