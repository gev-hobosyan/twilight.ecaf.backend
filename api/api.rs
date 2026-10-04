use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct APIError {
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

pub enum APIResponse<T> where T: Serialize {
    OK,
    Created(T),
    Patched(T),
    Json(T),
    Error(StatusCode, APIError)
}

impl<T> IntoResponse for APIResponse<T> where T: Serialize {
    fn into_response(self) -> Response {
        match self {
            APIResponse::Created(project) => {
                (
                    StatusCode::CREATED,
                    Json(project)
                ).into_response()
            }

            APIResponse::Patched(project) => {
                (
                    StatusCode::OK,
                    Json(project)
                ).into_response()
            }

            APIResponse::Json(project) => {
                (
                    StatusCode::OK,
                    Json(project)
                ).into_response()
            }

            APIResponse::Error(code, err) => {
                (
                    code,
                    Json(err)
                ).into_response()
            }
            APIResponse::OK => {
                StatusCode::OK.into_response()
            }
        }
    }
}