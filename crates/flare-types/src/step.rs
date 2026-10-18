use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: Uuid,
    pub run_id: Uuid,
    pub name: String,
    pub position: i32,
    pub status: StepStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}
