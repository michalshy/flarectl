use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::step::Step;

#[derive(Default, Copy, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum RunTrigger {
    #[default]
    External,
    Schedule,
    Ui,
    Missed,
}

#[derive(Copy, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
    Aborted,
}

#[derive(Copy, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "text", rename_all = "snake_case"))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum RunFailure {
    Reported,
    HeartbeatTimeout,
    Missed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Run {
    pub id: Uuid,
    pub job_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub trigger: RunTrigger,
    pub status: RunStatus,
    pub failure_reason: Option<RunFailure>,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRun {
    pub worker_id: Option<Uuid>,
    #[serde(default)]
    pub trigger: RunTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishRun {
    pub final_status: RunStatus,
    pub exit_code: Option<i32>,
}

#[derive(Deserialize)]
pub struct ListRuns {
    pub job_id: Option<Uuid>,
    pub status: Option<RunStatus>,
    pub limit: Option<i64>,
    pub before: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RunDetail {
    #[serde(flatten)]
    pub run: Run,
    pub steps: Vec<Step>,
}
