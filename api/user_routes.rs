use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use crate::api::APIResponse;
use crate::app_state::AppState;
use crate::bot::send_report;
use crate::entity::auth::prelude::User;
use crate::entity::auth::user;
use crate::entity::auth::user::Model;

async fn get_user_by_name(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>
) -> APIResponse<user::Model>{
    let user = User::find()
        .filter(user::Column::Name.eq(name))
        .one(&state.db)
        .await;

    match user {
        Ok(user) => {
            match user {
                Some(user) => {
                    APIResponse::Json(user)
                }
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "User not found".into()
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

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{name}", get(get_user_by_name))
}