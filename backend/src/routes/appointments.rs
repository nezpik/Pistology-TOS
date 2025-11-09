use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;
use validator::Validate;

use crate::models::{CreateTruckAppointment, TruckAppointment, UpdateTruckAppointment};

pub async fn get_appointments(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let appointments = sqlx::query_as::<_, TruckAppointment>(
        "SELECT * FROM truck_appointments ORDER BY appointment_time ASC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(appointments))
}

pub async fn get_appointments_by_company(
    pool: web::Data<SqlitePool>,
    company_name: web::Path<String>,
) -> Result<HttpResponse> {
    let appointments = sqlx::query_as::<_, TruckAppointment>(
        "SELECT * FROM truck_appointments WHERE trucking_company = ? ORDER BY appointment_time ASC",
    )
    .bind(company_name.as_str())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(appointments))
}

pub async fn create_appointment(
    pool: web::Data<SqlitePool>,
    data: web::Json<CreateTruckAppointment>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Insert appointment
    let result = sqlx::query(
        r#"
        INSERT INTO truck_appointments
        (trucking_company, driver_name, license_plate, appointment_time, status, container_id)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&data.trucking_company)
    .bind(&data.driver_name)
    .bind(&data.license_plate)
    .bind(&data.appointment_time)
    .bind(&data.status)
    .bind(&data.container_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the created appointment
    let appointment = sqlx::query_as::<_, TruckAppointment>(
        "SELECT * FROM truck_appointments WHERE id = ?",
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Created().json(appointment))
}

pub async fn update_appointment(
    pool: web::Data<SqlitePool>,
    appointment_id: web::Path<i64>,
    data: web::Json<UpdateTruckAppointment>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Update appointment
    let result = sqlx::query(
        r#"
        UPDATE truck_appointments SET
            trucking_company = ?,
            driver_name = ?,
            license_plate = ?,
            appointment_time = ?,
            status = ?,
            container_id = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(&data.trucking_company)
    .bind(&data.driver_name)
    .bind(&data.license_plate)
    .bind(&data.appointment_time)
    .bind(&data.status)
    .bind(&data.container_id)
    .bind(*appointment_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Appointment not found"
        })));
    }

    // Fetch the updated appointment
    let appointment = sqlx::query_as::<_, TruckAppointment>(
        "SELECT * FROM truck_appointments WHERE id = ?",
    )
    .bind(*appointment_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(appointment))
}

pub async fn delete_appointment(
    pool: web::Data<SqlitePool>,
    appointment_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let result = sqlx::query("DELETE FROM truck_appointments WHERE id = ?")
        .bind(*appointment_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Appointment not found"
        })));
    }

    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/appointments")
            .route("", web::get().to(get_appointments))
            .route("", web::post().to(create_appointment))
            .route("/company/{name}", web::get().to(get_appointments_by_company))
            .route("/{id}", web::put().to(update_appointment))
            .route("/{id}", web::delete().to(delete_appointment)),
    );
}
