use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DamageReport {
    pub id: i64,
    pub description: String,
    pub reported_by: String,
    pub photos: Option<String>, // JSON array stored as string
    pub container_id: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DamageReportWithPhotos {
    pub id: i64,
    pub description: String,
    pub reported_by: String,
    pub photos: Vec<String>,
    pub container_id: String,
    pub created_at: String,
}

impl From<DamageReport> for DamageReportWithPhotos {
    fn from(report: DamageReport) -> Self {
        let photos = report.photos
            .as_ref()
            .and_then(|p| serde_json::from_str(p).ok())
            .unwrap_or_default();

        Self {
            id: report.id,
            description: report.description,
            reported_by: report.reported_by,
            photos,
            container_id: report.container_id,
            created_at: report.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateDamageReport {
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub reported_by: String,
    pub photos: Option<Vec<String>>,
}
