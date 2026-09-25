mod entity;

use axum::Router;
use axum::routing::get;
use sea_orm::{ColIdx, Database, DatabaseConnection, EntityTrait};
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::Error;
use crate::entity::prelude::Users;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let router = Router::new();

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}
