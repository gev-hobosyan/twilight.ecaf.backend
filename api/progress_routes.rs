use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use hyper::StatusCode;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, Order, QueryFilter, QueryOrder, Set};
use serde::Deserialize;
use crate::api::APIResponse;
use crate::app_state::AppState;
use crate::bot::send_report;
use crate::entity::public::prelude::Progress;
use crate::entity::public::progress;
use crate::entity::public::progress::Model;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct ProgressBody {
    project_id: i32,
    date: String,
    description: String,
    image: String
}

async fn get_progress(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> APIResponse<Vec<progress::Model>> {
    let progress = Progress::find()
        .filter(progress::Column::ProjectId.eq(id))
        .order_by(progress::Column::Date, Order::Asc)
        .all(&state.db)
        .await;

    match progress {
        Ok(progress) => {
            APIResponse::Json(progress)
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

async fn add_progress(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ProgressBody>
) -> APIResponse<progress::Model>{
    let progress = progress::ActiveModel {
        id: Default::default(),
        project_id: Set(body.project_id),
        date: Set(body.date),
        description: Set(body.description),
        image: Set(body.image),
    };

    let res = progress.insert(&state.db).await;

    match res {
        Ok(progress) => {
            APIResponse::Json(progress)
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

pub fn progress_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/progress/{id}", get(get_progress))
        .route("/progress", post(add_progress))
}