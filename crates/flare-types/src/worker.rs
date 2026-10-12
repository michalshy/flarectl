use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// New worker, represented by table in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub labels: serde_json::Value,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

/// Already registered worker, present in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredWorker {
    pub name: String,
    pub version: Option<String>,
}
