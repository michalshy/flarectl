use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// No hash type, since we don't give it out, insert it from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub command: Option<String>,
    pub required_labels: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
