use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;
use validator::Validate;

use crate::models::{CreateCustomsInspection, CustomsInspection};

pub async fn get_customs_inspections(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
) -> Result<HttpResponse> {
    let inspections = sqlx::query_as::<_, CustomsInspection>(
        "SELECT * FROM customs_inspections WHERE container_id = ?",
    )
    .bind(container_id.as_str())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(inspections))
}

pub async fn create_customs_inspection(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
    data: web::Json<CreateCustomsInspection>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Insert customs inspection
    let result = sqlx::query(
        "INSERT INTO customs_inspections (status, notes, inspected_by, container_id) VALUES (?, ?, ?, ?)",
    )
    .bind(&data.status)
    .bind(&data.notes)
    .bind(&data.inspected_by)
    .bind(container_id.as_str())
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the created inspection
    let inspection = sqlx::query_as::<_, CustomsInspection>(
        "SELECT * FROM customs_inspections WHERE id = ?",
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Created().json(inspection))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/customs")
            .route("/{container_id}", web::get().to(get_customs_inspections))
            .route("/{container_id}", web::post().to(create_customs_inspection)),
    );
}
