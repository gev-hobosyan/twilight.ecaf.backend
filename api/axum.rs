mod entity;
mod app_state;

use std::env;
use std::sync::Arc;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::post;
use sea_orm::{ActiveModelTrait, ColIdx, EntityTrait, Iden, Set};
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::Error;
use crate::app_state::AppState;
use dotenvy::dotenv;
use hyper::StatusCode;
use sea_orm::prelude::Uuid;
use serde::Deserialize;
use crate::entity::invitation::Column::Status;
use crate::entity::prelude::Project;
use crate::entity::project;

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
struct ProjectBody {
    name: String,
    description: String,
    story: String,
    #[serde(rename="useAI")]
    use_ai: bool,
    tech_stack: String,
    notes: Option<String>,
    owner: Option<Uuid>
}

enum ProjectAPIResponse {
    Created(Project),

}

impl IntoResponse for ProjectAPIResponse {
    fn into_response(self) -> Response {
        match self {
            ProjectAPIResponse::Created(project) => {
                (
                    StatusCode::CREATED,
                    Json(project)
                ).into_response()
            }
        }
    }
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ProjectBody>
) -> ProjectAPIResponse {
    let project = project::ActiveModel {
        id: Default::default(),
        name: Set(body.name),
        description: Set(body.description),
        story: Set(body.story),
        use_ai: Set(body.use_ai),
        tech_stack: Set(body.tech_stack),
        notes: Set(body.notes),
        owner: Set(body.owner),
    };

    let project = project.insert(&state.db).await.unwrap();

    ProjectAPIResponse::Created(project)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().expect(".env file not found");

    let db_url = env::var("NEON_DATABASE_URL").unwrap();

    let state = Arc::new(AppState::new(db_url).await);

    let project_routes = Router::new()
        .route("/project", post(create_project));

    let router = Router::new()
        .merge(project_routes)
        .with_state(state);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}
