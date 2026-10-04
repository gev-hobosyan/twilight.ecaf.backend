use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, Set};
use serde::{Deserialize, Serialize};
use crate::api::{APIError, APIResponse};
use crate::app_state::AppState;
use crate::entity::public::prelude::Project;
use crate::entity::public::project;

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

async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ProjectBody>
) -> APIResponse<Project> {
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
            APIResponse::Created(project)
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> APIResponse<Project> {
    let project = project::Entity::find_by_id(id).one(&state.db).await;

    match project {
        Ok(res) => {
            match res {
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Project not found".into()
                    )
                }
                Some(project) => {
                    APIResponse::Json(project)
                }
            }
        }
        Err(err) => {
            APIResponse::Error(
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
) -> APIResponse<Project> {
    let result = crate::entity::public::project::Entity::find_by_id(id).one(&state.db).await;

    match result {
        Ok(project) => {
            match project {
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Project not found".into()
                    )
                }
                Some(project) => {
                    let project: project::ActiveModel = project.into();

                    let project = body.merge(project).update(&state.db).await;

                    match project {
                        Ok(project) => {
                            APIResponse::Patched(project)
                        }
                        Err(_) => {
                            APIResponse::Error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Connection failed".into()
                            )
                        }
                    }
                }
            }
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> APIResponse<Project> {
    let project = project::Entity::find_by_id(id).one(&state.db).await;

    match project {
        Ok(project) => {
            match project {
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Project not found".into()
                    )
                }
                Some(project) => {
                    let res = project.delete(&state.db).await;

                    APIResponse::OK
                }
            }
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

pub fn project_routes() -> Router<Arc<AppState>>{
    Router::new()
        .route("/project", post(create_project))
        .route("/project/{id}", get(get_project).patch(update_project).delete(delete_project))
}