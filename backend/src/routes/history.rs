use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;
use validator::Validate;

use crate::models::{CreateHistory, History};

pub async fn get_history(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
) -> Result<HttpResponse> {
    let history = sqlx::query_as::<_, History>(
        "SELECT * FROM history WHERE container_id = ? ORDER BY created_at DESC",
    )
    .bind(container_id.as_str())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(history))
}

pub async fn create_history(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
    data: web::Json<CreateHistory>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Insert history entry
    let result = sqlx::query(
        "INSERT INTO history (event, description, container_id) VALUES (?, ?, ?)",
    )
    .bind(&data.event)
    .bind(&data.description)
    .bind(container_id.as_str())
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the created history entry
    let history = sqlx::query_as::<_, History>(
        "SELECT * FROM history WHERE id = ?",
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Created().json(history))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/history")
            .route("/{container_id}", web::get().to(get_history))
            .route("/{container_id}", web::post().to(create_history)),
    );
}
