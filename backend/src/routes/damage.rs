use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;
use validator::Validate;

use crate::models::{CreateDamageReport, DamageReport, DamageReportWithPhotos};

pub async fn get_damage_reports(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
) -> Result<HttpResponse> {
    let reports = sqlx::query_as::<_, DamageReport>(
        "SELECT * FROM damage_reports WHERE container_id = ?",
    )
    .bind(container_id.as_str())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Convert to reports with parsed photos
    let reports_with_photos: Vec<DamageReportWithPhotos> = reports
        .into_iter()
        .map(|r| r.into())
        .collect();

    Ok(HttpResponse::Ok().json(reports_with_photos))
}

pub async fn create_damage_report(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
    data: web::Json<CreateDamageReport>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Serialize photos to JSON
    let photos_json = data.photos.as_ref().map(|p| {
        serde_json::to_string(p).unwrap_or_else(|_| "[]".to_string())
    });

    // Insert damage report
    let result = sqlx::query(
        "INSERT INTO damage_reports (description, reported_by, photos, container_id) VALUES (?, ?, ?, ?)",
    )
    .bind(&data.description)
    .bind(&data.reported_by)
    .bind(&photos_json)
    .bind(container_id.as_str())
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the created report
    let report = sqlx::query_as::<_, DamageReport>(
        "SELECT * FROM damage_reports WHERE id = ?",
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Created().json(DamageReportWithPhotos::from(report)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/damage")
            .route("/{container_id}", web::get().to(get_damage_reports))
            .route("/{container_id}", web::post().to(create_damage_report)),
    );
}
