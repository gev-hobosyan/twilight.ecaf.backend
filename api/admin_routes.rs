use std::env;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Router;
use axum::routing::get;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use sea_orm::prelude::Uuid;
use crate::api::APIResponse;
use crate::app_state::AppState;
use crate::entity::auth::prelude::User;
use crate::entity::auth::user;
use crate::entity::auth::user::Model;
use crate::entity::public::prelude::{Project, Team};
use crate::entity::public::{project, team};

fn check_api_key(headers: &HeaderMap, ) -> bool {
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());

    match api_key {
        Some(api_key) => {
            let original = env::var("ADMIN_API_KEY").unwrap();

            if api_key != original {
                return false;
            }

            true
        }
        None => {
            false
        }
    }
}

async fn get_participants(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<Vec<user::Model>> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let users = User::find()
        .filter(user::Column::Role.eq("user"))
        .all(&state.db)
        .await;

    if let Err(_) = users {
        return APIResponse::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Connection Failed".into()
        )
    }

    APIResponse::Json(users.unwrap())
}

async fn get_projects(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<Vec<Project>>{
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let projects = project::Entity::find().all(&state.db).await;

    if let Err(_) = projects {
        return APIResponse::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Connection failed".into()
        )
    }

    APIResponse::Json(projects.unwrap())
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>
) -> APIResponse<user::Model> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        );
    }

    let user = User::find_by_id(id)
        .one(&state.db).await;

    match user {
        Ok(user) => {
            match user {
                Some(user) => {
                    APIResponse::Json(user)
                }
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Not found".into()
                    )
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

async fn get_teams(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<Vec<Team>> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let teams = team::Entity::find()
        .all(&state.db).await;

    if let Err(_) = teams {
        return APIResponse::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Connection failed".into()
        )
    }

    APIResponse::Json(teams.unwrap())
}

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admin/users", get(get_participants))
        .route("/admin/projects", get(get_projects))
        .route("/admin/user/{id}", get(get_user))
        .route("/admin/teams", get(get_teams))
}