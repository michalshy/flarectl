use axum::{Json, http::StatusCode, response::IntoResponse};
use flare_types::error::{ErrorCode, FlareErrorResponse};

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Conflict,
    Unauthorized,
    Validation(&'static str),
    Db(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, msg) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, ErrorCode::NotFound, "not found"),
            AppError::Conflict => (
                StatusCode::CONFLICT,
                ErrorCode::Conflict,
                "conflicting unique value",
            ),
            AppError::Validation(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, ErrorCode::Validation, msg)
            }
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "unauthorized",
            ),
            AppError::Db(e) => {
                tracing::error!(?e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::Internal,
                    "internal error",
                )
            }
        };
        (
            status,
            Json(FlareErrorResponse {
                code,
                error: msg.to_string(),
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => AppError::NotFound,
            ref e
                if e.as_database_error()
                    .is_some_and(|e| e.is_foreign_key_violation()) =>
            {
                AppError::Validation("Referenced resource does not exist")
            }
            ref e
                if e.as_database_error()
                    .is_some_and(|e| e.is_unique_violation()) =>
            {
                AppError::Conflict
            }
            _ => AppError::Db(value),
        }
    }
}
