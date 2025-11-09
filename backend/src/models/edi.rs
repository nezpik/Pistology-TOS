use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// EDI Message
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EdiMessage {
    pub id: i64,
    pub message_type: String,
    pub content: String,
    pub container_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateEdiMessage {
    #[validate(custom(function = "validate_message_type"))]
    pub message_type: String,
    #[validate(length(min = 1))]
    pub content: String,
}

fn validate_message_type(message_type: &str) -> Result<(), validator::ValidationError> {
    match message_type {
        "BAPLIE" | "COARRI" | "CODECO" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_message_type")),
    }
}

// BAPLIE Message
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BaplieMessage {
    pub id: i64,
    pub edi_message_id: i64,
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BaplieContainer {
    pub id: i64,
    pub baplie_message_id: i64,
    pub container_id: String,
    pub bay: Option<String>,
    pub row: Option<String>,
    pub tier: Option<String>,
    pub size: Option<String>,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub container_type: Option<String>,
    pub weight: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaplieMessageWithContainers {
    pub id: i64,
    pub edi_message_id: i64,
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub containers: Vec<BaplieContainer>,
}

// COARRI Message
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CoarriMessage {
    pub id: i64,
    pub edi_message_id: i64,
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CoarriMovement {
    pub id: i64,
    pub coarri_message_id: i64,
    pub container_id: String,
    pub movement_type: Option<String>,
    pub stowage_location: Option<String>,
    pub iso_container_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoarriMessageWithMovements {
    pub id: i64,
    pub edi_message_id: i64,
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
    pub movements: Vec<CoarriMovement>,
}

// CODECO Message
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CodecoMessage {
    pub id: i64,
    pub edi_message_id: i64,
    pub gate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CodecoMovement {
    pub id: i64,
    pub codeco_message_id: i64,
    pub container_id: String,
    pub movement_type: Option<String>,
    pub truck_license_plate: Option<String>,
    pub iso_container_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodecoMessageWithMovements {
    pub id: i64,
    pub edi_message_id: i64,
    pub gate: Option<String>,
    pub movements: Vec<CodecoMovement>,
}

// Combined EDI response
#[derive(Debug, Serialize, Deserialize)]
pub struct EdiMessageResponse {
    pub id: i64,
    pub message_type: String,
    pub content: String,
    pub container_id: Option<String>,
    pub created_at: String,
    pub baplie_message: Option<BaplieMessageWithContainers>,
    pub coarri_message: Option<CoarriMessageWithMovements>,
    pub codeco_message: Option<CodecoMessageWithMovements>,
}

// Parsed data structures for EDI parsers
#[derive(Debug, Clone)]
pub struct ParsedBaplieData {
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub containers: Vec<ParsedBaplieContainer>,
}

#[derive(Debug, Clone)]
pub struct ParsedBaplieContainer {
    pub container_id: String,
    pub bay: Option<String>,
    pub row: Option<String>,
    pub tier: Option<String>,
    pub size: Option<String>,
    pub container_type: Option<String>,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ParsedCoarriData {
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
    pub movements: Vec<ParsedCoarriMovement>,
}

#[derive(Debug, Clone)]
pub struct ParsedCoarriMovement {
    pub container_id: String,
    pub movement_type: Option<String>,
    pub stowage_location: Option<String>,
    pub iso_container_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedCodecoData {
    pub gate: Option<String>,
    pub movements: Vec<ParsedCodecoMovement>,
}

#[derive(Debug, Clone)]
pub struct ParsedCodecoMovement {
    pub container_id: String,
    pub movement_type: Option<String>,
    pub truck_license_plate: Option<String>,
    pub iso_container_type: Option<String>,
}
