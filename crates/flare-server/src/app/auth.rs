use axum::{extract::FromRequestParts, http::header::AUTHORIZATION};
use rand::Rng;
use uuid::Uuid;

use crate::app::{AppState, error::AppError};

pub struct AuthJob {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthJob {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, AppError> {
        let token: &str = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split_once(' '))
            .and_then(|(scheme, token)| scheme.eq_ignore_ascii_case("bearer").then_some(token))
            .ok_or(AppError::Unauthorized)?;

        let id = sqlx::query_scalar!("SELECT id FROM jobs WHERE token = $1", token)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(AuthJob { id })
    }
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    format!("flare_{}", hex::encode(bytes))
}
