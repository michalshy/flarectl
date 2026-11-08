use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A registered worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    #[cfg_attr(feature = "utoipa", schema(required = true))]
    pub version: Option<String>,
    pub labels: serde_json::Value,
    pub registered_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

/// Worker registration details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RegisterWorker {
    pub name: String,
    pub version: Option<String>,
}
