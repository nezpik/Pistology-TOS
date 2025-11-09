use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct History {
    pub id: i64,
    pub event: String,
    pub description: Option<String>,
    pub container_id: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateHistory {
    #[validate(length(min = 1))]
    pub event: String,
    pub description: Option<String>,
}
