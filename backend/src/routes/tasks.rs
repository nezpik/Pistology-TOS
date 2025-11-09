use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;
use validator::Validate;

use crate::models::{CreateTask, Task, UpdateTask};

pub async fn get_tasks(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks ORDER BY created_at DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(tasks))
}

pub async fn create_task(
    pool: web::Data<SqlitePool>,
    data: web::Json<CreateTask>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Insert task
    let result = sqlx::query(
        "INSERT INTO tasks (title, description, status, assignee, container_id) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&data.title)
    .bind(&data.description)
    .bind(&data.status)
    .bind(&data.assignee)
    .bind(&data.container_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the created task
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE id = ?",
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Created().json(task))
}

pub async fn update_task(
    pool: web::Data<SqlitePool>,
    task_id: web::Path<i64>,
    data: web::Json<UpdateTask>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Update task
    let result = sqlx::query(
        r#"
        UPDATE tasks SET
            title = ?,
            description = ?,
            status = ?,
            assignee = ?,
            container_id = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(&data.title)
    .bind(&data.description)
    .bind(&data.status)
    .bind(&data.assignee)
    .bind(&data.container_id)
    .bind(*task_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        })));
    }

    // Fetch the updated task
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE id = ?",
    )
    .bind(*task_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(task))
}

pub async fn delete_task(
    pool: web::Data<SqlitePool>,
    task_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(*task_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        })));
    }

    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tasks")
            .route("", web::get().to(get_tasks))
            .route("", web::post().to(create_task))
            .route("/{id}", web::put().to(update_task))
            .route("/{id}", web::delete().to(delete_task)),
    );
}
