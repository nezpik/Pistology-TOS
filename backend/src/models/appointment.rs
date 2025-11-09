use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TruckAppointment {
    pub id: i64,
    pub trucking_company: String,
    pub driver_name: String,
    pub license_plate: String,
    pub appointment_time: String,
    pub status: String,
    pub container_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTruckAppointment {
    #[validate(length(min = 1))]
    pub trucking_company: String,
    #[validate(length(min = 1))]
    pub driver_name: String,
    #[validate(length(min = 1))]
    pub license_plate: String,
    #[validate(length(min = 1))]
    pub appointment_time: String,
    #[validate(custom(function = "validate_appointment_status"))]
    pub status: String,
    pub container_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTruckAppointment {
    #[validate(length(min = 1))]
    pub trucking_company: String,
    #[validate(length(min = 1))]
    pub driver_name: String,
    #[validate(length(min = 1))]
    pub license_plate: String,
    #[validate(length(min = 1))]
    pub appointment_time: String,
    #[validate(custom(function = "validate_appointment_status"))]
    pub status: String,
    pub container_id: Option<String>,
}

fn validate_appointment_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "SCHEDULED" | "COMPLETED" | "CANCELLED" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_status")),
    }
}
