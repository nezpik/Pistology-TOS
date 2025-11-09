use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub assignee: Option<String>,
    pub container_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTask {
    #[validate(length(min = 1))]
    pub title: String,
    pub description: Option<String>,
    #[validate(custom(function = "validate_task_status"))]
    pub status: String,
    pub assignee: Option<String>,
    pub container_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTask {
    #[validate(length(min = 1))]
    pub title: String,
    pub description: Option<String>,
    #[validate(custom(function = "validate_task_status"))]
    pub status: String,
    pub assignee: Option<String>,
    pub container_id: Option<String>,
}

fn validate_task_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "PENDING" | "IN_PROGRESS" | "COMPLETED" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_status")),
    }
}
