use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use hyper::StatusCode;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, Set};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use crate::api::APIResponse;
use crate::app_state::AppState;
use crate::entity::auth::prelude::User;
use crate::entity::auth::user;
use crate::entity::public::prelude::{Team, TeamEntity};
use crate::entity::public::team;

#[derive(Deserialize)]
struct TeamBody{
    name: String,
    members: Option<Vec<Uuid>>
}

#[derive(Deserialize)]
struct TeamBodyOpt{
    name: Option<String>,
    members: Option<Vec<Uuid>>
}

impl TeamBodyOpt {
    fn merge(self, mut team: team::ActiveModel) -> team::ActiveModel {
        if let Some(name) = self.name {
            team.name = Set(Some(name))
        }

        team
    }
}

#[derive(Serialize)]
struct TeamResponse {
    id: i32,
    name: String,
    members: Vec<Uuid>
}

async fn get_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> APIResponse<TeamResponse>{
    let res = TeamEntity::find_by_id(id).one(&state.db).await;

    match res {
        Ok(team_opt) => {
            match team_opt {
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Team not found".into()
                    )
                }
                Some(team) => {
                    let members: Vec<Uuid> = User::find()
                        .filter(user::Column::Team.eq(team.id))
                        .select_only()
                        .column(user::Column::Id)
                        .into_tuple()
                        .all(&state.db)
                        .await
                        .unwrap_or(vec![]);

                    APIResponse::Json(TeamResponse {
                        id: team.id,
                        name: team.name.unwrap(),
                        members
                    })
                }
            }
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection Failed".into()
            )
        }
    }
}

async fn create_team(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TeamBody>
) -> APIResponse<Team> {
    let members = body.members.unwrap_or(vec![]);

    if members.len() > 3 {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "Teams can't have more that 3 people".into()
        )
    }

    let team = team::ActiveModel {
        id: Default::default(),
        name: Set(Option::from(body.name))
    };

    let team = team.insert(&state.db).await;

    match team {
        Ok(team) => {
            for member in members {
                let user = User::find_by_id(member).one(&state.db).await;

                match user {
                    Ok(user) => {
                        if let Some(user) = user {
                            let mut user: user::ActiveModel = user.into();

                            user.team = Set(Option::from(team.id));

                            let _ = user.update(&state.db).await;
                        }
                    }
                    Err(_) => {
                        return APIResponse::Error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Connection failed".into()
                        )
                    }
                }
            }

            APIResponse::Json(team)
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn update_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(body): Json<TeamBodyOpt>
) -> APIResponse<Team> {
    let res = TeamEntity::find_by_id(id).one(&state.db).await;

    match res {
        Ok(res) => {
            match res {
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Team not found".into()
                    )
                }
                Some(team) => {
                    let team: team::ActiveModel = team.into();

                    match body.merge(team).update(&state.db).await {
                        Ok(team) => {
                            APIResponse::Patched(team)
                        }
                        Err(_) => {
                            APIResponse::Error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Connection Failed".into()
                            )
                        }
                    }
                }
            }
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection Failed".into()
            )
        }
    }
}

pub fn team_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/team", post(create_team))
        .route("/team/{id}", get(get_team).patch(update_team))
}