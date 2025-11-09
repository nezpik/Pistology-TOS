use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CustomsInspection {
    pub id: i64,
    pub status: String,
    pub notes: Option<String>,
    pub inspected_by: Option<String>,
    pub container_id: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateCustomsInspection {
    #[validate(custom(function = "validate_inspection_status"))]
    pub status: String,
    pub notes: Option<String>,
    pub inspected_by: Option<String>,
}

fn validate_inspection_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "PENDING" | "IN_PROGRESS" | "COMPLETED" | "FAILED" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_status")),
    }
}
