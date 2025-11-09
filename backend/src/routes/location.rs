use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;
use validator::Validate;

use crate::models::{Location, UpdateLocation};

pub async fn get_location(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
) -> Result<HttpResponse> {
    let location = sqlx::query_as::<_, Location>(
        "SELECT * FROM locations WHERE container_id = ?",
    )
    .bind(container_id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match location {
        Some(loc) => Ok(HttpResponse::Ok().json(loc)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Location not found"
        }))),
    }
}

pub async fn update_location(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
    data: web::Json<UpdateLocation>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Upsert location
    sqlx::query(
        r#"
        INSERT INTO locations (container_id, lat, lng)
        VALUES (?, ?, ?)
        ON CONFLICT(container_id) DO UPDATE SET
            lat = excluded.lat,
            lng = excluded.lng,
            updated_at = datetime('now')
        "#,
    )
    .bind(container_id.as_str())
    .bind(data.lat)
    .bind(data.lng)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the updated location
    let location = sqlx::query_as::<_, Location>(
        "SELECT * FROM locations WHERE container_id = ?",
    )
    .bind(container_id.as_str())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(location))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/location")
            .route("/{container_id}", web::get().to(get_location))
            .route("/{container_id}", web::post().to(update_location)),
    );
}
