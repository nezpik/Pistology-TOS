use actix_web::{web, HttpResponse, Result};
use sqlx::{SqlitePool, Transaction, Sqlite};
use validator::Validate;

use crate::models::edi::*;
use crate::services::edi::{parse_baplie, parse_coarri, parse_codeco};

pub async fn get_edi_messages(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
) -> Result<HttpResponse> {
    let edi_messages = sqlx::query_as::<_, EdiMessage>(
        "SELECT * FROM edi_messages WHERE container_id = ?",
    )
    .bind(container_id.as_str())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let mut responses = Vec::new();

    for msg in edi_messages {
        let mut response = EdiMessageResponse {
            id: msg.id,
            message_type: msg.message_type.clone(),
            content: msg.content.clone(),
            container_id: msg.container_id.clone(),
            created_at: msg.created_at.clone(),
            baplie_message: None,
            coarri_message: None,
            codeco_message: None,
        };

        match msg.message_type.as_str() {
            "BAPLIE" => {
                if let Ok(Some(baplie)) = fetch_baplie_with_containers(pool.get_ref(), msg.id).await {
                    response.baplie_message = Some(baplie);
                }
            }
            "COARRI" => {
                if let Ok(Some(coarri)) = fetch_coarri_with_movements(pool.get_ref(), msg.id).await {
                    response.coarri_message = Some(coarri);
                }
            }
            "CODECO" => {
                if let Ok(Some(codeco)) = fetch_codeco_with_movements(pool.get_ref(), msg.id).await {
                    response.codeco_message = Some(codeco);
                }
            }
            _ => {}
        }

        responses.push(response);
    }

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn create_edi_message(
    pool: web::Data<SqlitePool>,
    container_id: web::Path<String>,
    data: web::Json<CreateEdiMessage>,
) -> Result<HttpResponse> {
    // Validate input
    data.validate().map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Validation error: {}", e))
    })?;

    // Start transaction
    let mut tx = pool.begin().await.map_err(|e| {
        log::error!("Transaction error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Insert EDI message
    let edi_result = sqlx::query(
        "INSERT INTO edi_messages (message_type, content, container_id) VALUES (?, ?, ?)",
    )
    .bind(&data.message_type)
    .bind(&data.content)
    .bind(container_id.as_str())
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let edi_message_id = edi_result.last_insert_rowid();

    // Parse and store based on message type
    match data.message_type.as_str() {
        "BAPLIE" => {
            let parsed = parse_baplie(&data.content).map_err(|e| {
                actix_web::error::ErrorBadRequest(format!("BAPLIE parsing error: {}", e))
            })?;
            store_baplie(&mut tx, edi_message_id, parsed).await?;
        }
        "COARRI" => {
            let parsed = parse_coarri(&data.content).map_err(|e| {
                actix_web::error::ErrorBadRequest(format!("COARRI parsing error: {}", e))
            })?;
            store_coarri(&mut tx, edi_message_id, parsed).await?;
        }
        "CODECO" => {
            let parsed = parse_codeco(&data.content).map_err(|e| {
                actix_web::error::ErrorBadRequest(format!("CODECO parsing error: {}", e))
            })?;
            store_codeco(&mut tx, edi_message_id, parsed).await?;
        }
        _ => {
            return Err(actix_web::error::ErrorBadRequest("Invalid message type"));
        }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| {
        log::error!("Transaction commit error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Fetch the created message with parsed data
    let edi_message = sqlx::query_as::<_, EdiMessage>(
        "SELECT * FROM edi_messages WHERE id = ?",
    )
    .bind(edi_message_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let mut response = EdiMessageResponse {
        id: edi_message.id,
        message_type: edi_message.message_type.clone(),
        content: edi_message.content.clone(),
        container_id: edi_message.container_id.clone(),
        created_at: edi_message.created_at.clone(),
        baplie_message: None,
        coarri_message: None,
        codeco_message: None,
    };

    // Fetch parsed data
    match data.message_type.as_str() {
        "BAPLIE" => {
            if let Ok(Some(baplie)) = fetch_baplie_with_containers(pool.get_ref(), edi_message_id).await {
                response.baplie_message = Some(baplie);
            }
        }
        "COARRI" => {
            if let Ok(Some(coarri)) = fetch_coarri_with_movements(pool.get_ref(), edi_message_id).await {
                response.coarri_message = Some(coarri);
            }
        }
        "CODECO" => {
            if let Ok(Some(codeco)) = fetch_codeco_with_movements(pool.get_ref(), edi_message_id).await {
                response.codeco_message = Some(codeco);
            }
        }
        _ => {}
    }

    Ok(HttpResponse::Created().json(response))
}

// Helper functions

async fn store_baplie(
    tx: &mut Transaction<'_, Sqlite>,
    edi_message_id: i64,
    data: ParsedBaplieData,
) -> Result<()> {
    // Insert BAPLIE message
    let baplie_result = sqlx::query(
        r#"
        INSERT INTO baplie_messages (edi_message_id, vessel_name, voyage_number, port_of_loading, port_of_discharge)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(edi_message_id)
    .bind(&data.vessel_name)
    .bind(&data.voyage_number)
    .bind(&data.port_of_loading)
    .bind(&data.port_of_discharge)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let baplie_message_id = baplie_result.last_insert_rowid();

    // Insert containers
    for container in data.containers {
        sqlx::query(
            r#"
            INSERT INTO baplie_containers (baplie_message_id, container_id, bay, row, tier, size, type, weight)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(baplie_message_id)
        .bind(&container.container_id)
        .bind(&container.bay)
        .bind(&container.row)
        .bind(&container.tier)
        .bind(&container.size)
        .bind(&container.container_type)
        .bind(container.weight)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    }

    Ok(())
}

async fn store_coarri(
    tx: &mut Transaction<'_, Sqlite>,
    edi_message_id: i64,
    data: ParsedCoarriData,
) -> Result<()> {
    // Insert COARRI message
    let coarri_result = sqlx::query(
        "INSERT INTO coarri_messages (edi_message_id, vessel_name, voyage_number) VALUES (?, ?, ?)",
    )
    .bind(edi_message_id)
    .bind(&data.vessel_name)
    .bind(&data.voyage_number)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let coarri_message_id = coarri_result.last_insert_rowid();

    // Insert movements
    for movement in data.movements {
        sqlx::query(
            r#"
            INSERT INTO coarri_movements (coarri_message_id, container_id, movement_type, stowage_location, iso_container_type)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(coarri_message_id)
        .bind(&movement.container_id)
        .bind(&movement.movement_type)
        .bind(&movement.stowage_location)
        .bind(&movement.iso_container_type)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    }

    Ok(())
}

async fn store_codeco(
    tx: &mut Transaction<'_, Sqlite>,
    edi_message_id: i64,
    data: ParsedCodecoData,
) -> Result<()> {
    // Insert CODECO message
    let codeco_result = sqlx::query(
        "INSERT INTO codeco_messages (edi_message_id, gate) VALUES (?, ?)",
    )
    .bind(edi_message_id)
    .bind(&data.gate)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let codeco_message_id = codeco_result.last_insert_rowid();

    // Insert movements
    for movement in data.movements {
        sqlx::query(
            r#"
            INSERT INTO codeco_movements (codeco_message_id, container_id, movement_type, truck_license_plate, iso_container_type)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(codeco_message_id)
        .bind(&movement.container_id)
        .bind(&movement.movement_type)
        .bind(&movement.truck_license_plate)
        .bind(&movement.iso_container_type)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    }

    Ok(())
}

async fn fetch_baplie_with_containers(
    pool: &SqlitePool,
    edi_message_id: i64,
) -> Result<Option<BaplieMessageWithContainers>> {
    let baplie = sqlx::query_as::<_, BaplieMessage>(
        "SELECT * FROM baplie_messages WHERE edi_message_id = ?",
    )
    .bind(edi_message_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if let Some(msg) = baplie {
        let containers = sqlx::query_as::<_, BaplieContainer>(
            "SELECT * FROM baplie_containers WHERE baplie_message_id = ?",
        )
        .bind(msg.id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

        Ok(Some(BaplieMessageWithContainers {
            id: msg.id,
            edi_message_id: msg.edi_message_id,
            vessel_name: msg.vessel_name,
            voyage_number: msg.voyage_number,
            port_of_loading: msg.port_of_loading,
            port_of_discharge: msg.port_of_discharge,
            containers,
        }))
    } else {
        Ok(None)
    }
}

async fn fetch_coarri_with_movements(
    pool: &SqlitePool,
    edi_message_id: i64,
) -> Result<Option<CoarriMessageWithMovements>> {
    let coarri = sqlx::query_as::<_, CoarriMessage>(
        "SELECT * FROM coarri_messages WHERE edi_message_id = ?",
    )
    .bind(edi_message_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if let Some(msg) = coarri {
        let movements = sqlx::query_as::<_, CoarriMovement>(
            "SELECT * FROM coarri_movements WHERE coarri_message_id = ?",
        )
        .bind(msg.id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

        Ok(Some(CoarriMessageWithMovements {
            id: msg.id,
            edi_message_id: msg.edi_message_id,
            vessel_name: msg.vessel_name,
            voyage_number: msg.voyage_number,
            movements,
        }))
    } else {
        Ok(None)
    }
}

async fn fetch_codeco_with_movements(
    pool: &SqlitePool,
    edi_message_id: i64,
) -> Result<Option<CodecoMessageWithMovements>> {
    let codeco = sqlx::query_as::<_, CodecoMessage>(
        "SELECT * FROM codeco_messages WHERE edi_message_id = ?",
    )
    .bind(edi_message_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if let Some(msg) = codeco {
        let movements = sqlx::query_as::<_, CodecoMovement>(
            "SELECT * FROM codeco_movements WHERE codeco_message_id = ?",
        )
        .bind(msg.id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

        Ok(Some(CodecoMessageWithMovements {
            id: msg.id,
            edi_message_id: msg.edi_message_id,
            gate: msg.gate,
            movements,
        }))
    } else {
        Ok(None)
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/edi")
            .route("/{container_id}", web::get().to(get_edi_messages))
            .route("/{container_id}", web::post().to(create_edi_message)),
    );
}
