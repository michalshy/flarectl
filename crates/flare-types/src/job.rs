use crate::common::empty_object;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub command: Option<String>,
    pub required_labels: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Secret {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJob {
    pub key: String,
    pub name: String,
    pub command: Option<String>,
    #[serde(default = "empty_object")]
    pub required_labels: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreatedJob {
    pub id: Uuid,
    pub token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenameJob {
    pub name: String,
}
