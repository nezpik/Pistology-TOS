mod models;
mod routes;
mod services;
mod middleware;

use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:/app/data/prod.db".to_string());

    log::info!("Connecting to database: {}", database_url);

    // Create database pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    log::info!("Database pool created successfully");

    // Run migrations (create tables from schema.sql)
    log::info!("Running database migrations...");
    sqlx::query(&std::fs::read_to_string("schema.sql").expect("Failed to read schema.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Database migrations completed");

    log::info!("Starting server on 0.0.0.0:3001");

    // Create and run HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            // App data
            .app_data(web::Data::new(pool.clone()))
            // Middleware
            .wrap(actix_middleware::Logger::default())
            .wrap(actix_middleware::Compress::default())
            .wrap(cors)
            .wrap(actix_middleware::DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("X-Frame-Options", "DENY"))
                .add(("X-XSS-Protection", "1; mode=block"))
                .add(("Cache-Control", "public, max-age=300"))
            )
            // Root routes
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            // API routes
            .configure(routes::location::configure)
            .configure(routes::history::configure)
            .configure(routes::damage::configure)
            .configure(routes::customs::configure)
            .configure(routes::tasks::configure)
            .configure(routes::appointments::configure)
            .configure(routes::edi::configure)
    })
    .bind(("0.0.0.0", 3001))?
    .run()
    .await
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "name": "Pistology TOS API",
        "version": "2.0.0",
        "description": "Terminal Operating System REST API - Rust Edition",
        "status": "running"
    }))
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
