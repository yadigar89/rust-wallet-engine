use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Verilenler bazasi xetasi: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Xeta: {0}")]
    NotFound(String),

    #[error("Xeta: {0}")]
    BadRequest(String),

    #[error("Xeta: {0}")]
    Unauthorized(String),

    #[error("Daxili server xetasi: {0}")]
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message): (StatusCode, String) = match &self {
            AppError::DatabaseError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Verilenler bazasi xetasi: {}", err),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg.clone(),
            ),
        };

        let body: Json<Value> = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}