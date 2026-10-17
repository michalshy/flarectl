use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub enum AppError {
    NotFound,
    Db(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found"),
            AppError::Db(e) => {
                tracing::error!(?e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        };
        (status, Json(json!({"error": msg}))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::Db(value),
        }
    }
}
