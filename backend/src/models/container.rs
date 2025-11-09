use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Container {
    pub id: i64,
    pub container_id: String,
    pub bay: Option<String>,
    pub row: Option<String>,
    pub tier: Option<String>,
    pub size: Option<String>,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub container_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateContainer {
    pub container_id: String,
    pub bay: Option<String>,
    pub row: Option<String>,
    pub tier: Option<String>,
    pub size: Option<String>,
    #[serde(rename = "type")]
    pub container_type: Option<String>,
}
