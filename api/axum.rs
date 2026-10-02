mod entity;
mod app_state;

use std::env;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::{get, post};
use sea_orm::{ActiveModelTrait, ColIdx, DbErr, EntityTrait, Iden, ModelTrait, Set, Unchanged};
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;
use vercel_runtime::Error;
use crate::app_state::AppState;
use dotenvy::dotenv;
use hyper::StatusCode;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use crate::entity::public::prelude::Project;
use crate::entity::public::project;
use crate::entity::public::project::{Entity as ProjectEntity, Model};

#[derive(Serialize, Debug)]
struct APIError {
    message: String
}

impl APIError {
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}
impl From<&str> for APIError {
    fn from(value: &str) -> Self {
        APIError::new(value.into())
    }
}

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

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
struct ProjectBodyOpt {
    name: Option<String>,
    description: Option<String>,
    story: Option<String>,
    #[serde(rename="useAI")]
    use_ai: Option<bool>,
    tech_stack: Option<String>,
    notes: Option<String>,
    owner: Option<Uuid>
}

impl ProjectBodyOpt {
    pub fn merge(self, mut model: project::ActiveModel) -> project::ActiveModel{
        if let Some(name) = self.name {
            model.name = Set(name);
        }

        if let Some(description) = self.description {
            model.description = Set(description);
        }

        if let Some(story) = self.story {
            model.story = Set(story);
        }

        if let Some(use_ai) = self.use_ai {
            model.use_ai = Set(use_ai);
        }

        if let Some(tech_stack) = self.tech_stack {
            model.tech_stack = Set(tech_stack);
        }

        if let Some(notes) = self.notes {
            model.notes = Set(Some(notes));
        }

        if let Some(owner) = self.owner {
            model.owner = Set(Some(owner));
        }

        model
    }
}

enum ProjectAPIResponse {
    OK,
    Created(Project),
    Patched(Project),
    Json(Project),
    Error(StatusCode, APIError)
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

            ProjectAPIResponse::Patched(project) => {
                (
                    StatusCode::OK,
                    Json(project)
                ).into_response()
            }

            ProjectAPIResponse::Json(project) => {
                (
                    StatusCode::OK,
                    Json(project)
                ).into_response()
            }

            ProjectAPIResponse::Error(code, err) => {
                (
                    code,
                    Json(err)
                ).into_response()
            }
            ProjectAPIResponse::OK => {
                StatusCode::OK.into_response()
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

    let result = project.insert(&state.db).await;

    match result {
        Ok(project) => {
            ProjectAPIResponse::Created(project)
        }
        Err(_) => {
            ProjectAPIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> ProjectAPIResponse {
    let project = ProjectEntity::find_by_id(id).one(&state.db).await;

    match project {
        Ok(res) => {
            match res {
                None => {
                    ProjectAPIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Project not found".into()
                    )
                }
                Some(project) => {
                    ProjectAPIResponse::Json(project)
                }
            }
        }
        Err(err) => {
            ProjectAPIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(body): Json<ProjectBodyOpt>,
) -> ProjectAPIResponse {
    let result = ProjectEntity::find_by_id(id).one(&state.db).await;

    match result {
        Ok(project) => {
            match project {
                None => {
                    ProjectAPIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Project not found".into()
                    )
                }
                Some(project) => {
                    let project: project::ActiveModel = project.into();

                    let project = body.merge(project).update(&state.db).await;

                    match project {
                        Ok(project) => {
                            ProjectAPIResponse::Json(project)
                        }
                        Err(_) => {
                            ProjectAPIResponse::Error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Connection failed".into()
                            )
                        }
                    }
                }
            }
        }
        Err(_) => {
            ProjectAPIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> ProjectAPIResponse{
    let project = ProjectEntity::find_by_id(id).one(&state.db).await;

    match project {
        Ok(project) => {
            match project {
                None => {
                    ProjectAPIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Project not found".into()
                    )
                }
                Some(project) => {
                    let res = project.delete(&state.db).await;

                    ProjectAPIResponse::OK
                }
            }
        }
        Err(_) => {
            ProjectAPIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().expect(".env file not found");

    let db_url = env::var("NEON_DATABASE_URL").unwrap();

    let state = Arc::new(AppState::new(db_url).await);

    let project_routes = Router::new()
        .route("/project", post(create_project))
        .route("/project/{id}", get(get_project).patch(update_project).delete(delete_project));

    let router = Router::new()
        .merge(project_routes)
        .with_state(state);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}
