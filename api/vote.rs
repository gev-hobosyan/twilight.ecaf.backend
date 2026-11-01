use std::sync::Arc;
use axum::extract::{ws, State, WebSocketUpgrade};
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use futures_util::{SinkExt, StreamExt};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use crate::admin_routes::check_api_key;
use crate::api::APIResponse;
use crate::app_state::{AppState, VoteState};
use crate::entity::public::project;
use crate::vote_event::VoteEvent;

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse{
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    {
        let vote_state = state.vote.lock().await;

        if vote_state.is_waiting() {
            let _ = state.tx.send(VoteEvent::Waiting);
        }
    }

    let send_state = Arc::clone(&state);
    let recv_state = Arc::clone(&state);

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = (sender).send(msg.clone().into()).await;

            if let VoteEvent::Close = msg {
                let _ = sender.send(Message::Close(Some(CloseFrame {
                    code: ws::close_code::NORMAL,
                    reason: Utf8Bytes::from_static("Vote ended"),
                }))).await;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let _ = recv_state.tx.send(VoteEvent::Empty);
        }
    });

    tokio::select! {
        rv_a = (&mut recv_task) => {
            match rv_a {
                Ok(_) => {}
                Err(_) => {}
            }

            recv_task.abort();
        },
        rv_b = (&mut send_task) => {
            match rv_b {
                Ok(_) => {}
                Err(_) => {}
            }

            send_task.abort();
        }
    };
}

async fn start_vote(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<()> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let mut vote_state = state.vote.lock().await;

    if vote_state.is_waiting() {
        let projects = project::Entity::find()
            .all(&state.db)
            .await.unwrap_or(vec![]);

        let _ = state.tx.send(VoteEvent::Start(projects.clone()));

        *vote_state = VoteState::InProcess(projects);

        return APIResponse::OK;
    }

    APIResponse::Error(
        StatusCode::BAD_REQUEST,
        "Vote already in process".into()
    )
}

async fn vote(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UserVotes>
) -> APIResponse<()>{
    let votes = body.votes;
    let vote = state.vote.lock().await;

    match *vote {
        VoteState::NotStarted => {
            APIResponse::Error(
                StatusCode::BAD_REQUEST,
                "Vote not started yet".into()
            )
        }
        VoteState::InProcess(_) => {
            let _ = state.tx.send(VoteEvent::Vote(votes.0, votes.1, votes.2));

            APIResponse::OK
        }
    }
}

async fn close_vote(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<()> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let mut vote_state = state.vote.lock().await;

    *vote_state = VoteState::NotStarted;

    let _ = state.tx.send(VoteEvent::Close);

    APIResponse::OK
}

async fn get_project_to_vote(
    State(state): State<Arc<AppState>>
) -> APIResponse<Vec<project::Model>> {
    let vote = state.vote.lock().await;

    match *vote {
        VoteState::NotStarted => {
            APIResponse::Error(
                StatusCode::BAD_REQUEST,
                "Vote not started yet".into()
            )
        }
        VoteState::InProcess(_) => {
            APIResponse::Json(vote.get_projects())
        }
    }
}

pub fn vote_routes() -> Router<Arc<AppState>>{
    Router::new()
        .route("/vote/display", get(ws_handler))
        .route("/vote/start", get(start_vote))
        .route("/vote", post(vote).get(get_project_to_vote))
        .route("/vote/close", get(close_vote))
}

#[derive(Serialize, Deserialize)]
struct UserVotes {
    votes: (i32, i32, i32)
}