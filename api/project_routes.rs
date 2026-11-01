use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::routing::{get, post};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, ModelTrait, QuerySelect, Set};
use serde::{Deserialize};
use crate::api::{APIResponse};
use crate::app_state::AppState;
use crate::bot::send_report;
use crate::entity::auth::prelude::User;
use crate::entity::auth::user;
use crate::entity::public::prelude::{Project, TeamEntity};
use crate::entity::public::project;
use crate::entity::public::project::Model;

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
    github: Option<String>,
    tech_stack: Option<String>,
    notes: Option<String>,
    owner: Option<Uuid>,
    submitted: Option<bool>
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

        if let Some(submitted) = self.submitted {
            model.submitted = Set(submitted);
        }

        if let Some(github) = self.github {
            model.github = Set(Some(github));
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
        github: Set(Some("".to_string())),
        notes: Set(body.notes),
        owner: Set(body.owner),
        submitted: Default::default()
    };

    let result = project.insert(&state.db).await;

    match result {
        Ok(project) => {
            if let Some(owner) = body.owner {
                let team: Result<Option<i32>, DbErr> = User::find_by_id(owner)
                    .select_only()
                    .column(user::Column::Team)
                    .into_tuple()
                    .one(&state.db)
                    .await;

                if let Ok(team) = team {
                    match team {
                        None => {
                            let owner = User::find_by_id(owner)
                                .one(&state.db)
                                .await;

                            if let Ok(Some(owner)) = owner {
                                let mut owner: user::ActiveModel = owner.into();

                                owner.project = Set(Some(project.id));

                                let _ = owner.update(&state.db).await;
                            }
                        }
                        Some(team_id) => {
                            let members = User::find()
                                .filter(user::Column::Team.eq(team_id))
                                .all(&state.db)
                                .await
                                .unwrap_or(vec![]);

                            for member in members {
                                let mut member: user::ActiveModel = member.into();

                                member.project = Set(Some(project.id));

                                let _ = member.update(&state.db).await;
                            }
                        }
                    }
                }
            }

            APIResponse::Created(project)
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
            send_report(&state.bot, state.chat_id, err.to_string()).await;

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
    let result = project::Entity::find_by_id(id).one(&state.db).await;

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
        Err(err) => {
            send_report(&state.bot, state.chat_id, err.to_string()).await;

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
        Err(err) => {
            send_report(&state.bot, state.chat_id, err.to_string()).await;

            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn get_project_of_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>
) -> APIResponse<Project> {
    let project = project::Entity::find()
        .filter(project::Column::Owner.eq(Some(id)))
        .one(&state.db)
        .await;

    match project {
        Ok(project) => {
            match project {
                Some(project) => {
                    APIResponse::Json(project)
                }
                None => {
                    let user = User::find_by_id(id)
                        .one(&state.db)
                        .await;

                    match user {
                        Ok(user) => {
                            match user {
                                Some(user) => {
                                    match user.project {
                                        Some(project_id) => {
                                            let project = project::Entity::find_by_id(project_id)
                                                .one(&state.db)
                                                .await;

                                            match project {
                                                Ok(project) => {
                                                    match project {
                                                        Some(project) => {
                                                            APIResponse::Json(project)
                                                        }
                                                        None => {
                                                            APIResponse::Error(
                                                                StatusCode::NOT_FOUND,
                                                                "Project Not Found".into()
                                                            )
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
                                        None => {
                                            APIResponse::Error(
                                                StatusCode::NO_CONTENT,
                                                "User without project".into()
                                            )
                                        }
                                    }
                                }
                                None => {
                                    APIResponse::Error(
                                        StatusCode::NOT_FOUND,
                                        "Project Not Found".into()
                                    )
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
pub fn project_routes() -> Router<Arc<AppState>>{
    Router::new()
        .route("/project", post(create_project))
        .route("/project/{id}", get(get_project).patch(update_project).delete(delete_project))
        .route("/project/user/{id}", get(get_project_of_user))
}