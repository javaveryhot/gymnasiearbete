use std::io::Write;

use crate::{docker::docker_api::gcc_container, Result};
use axum::{extract::Multipart, http::StatusCode, Json};
use serde_json::{json, Value};
use tempfile::tempfile;
use tokio::{fs::File, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{
    ctx::Ctx, docker::docker_api::configure_and_run_secure_container,
    schema::session_tokens::user_uuid, Error,
};

pub async fn run_user_code(ctx: Ctx) -> Result<Json<Value>> {
    info!("Authenticated Successfully");
    // run demo code
    match configure_and_run_secure_container().await {
        Ok(()) => {
            info!("Successfully ran container");
            Ok(Json(json!({
                "message": "Successfully ran container"
            })))
        }
        Err(e) => {
            error!("Error running container: {}", e);
            Err(Error::InternalServerError)
        }
    }
}

async fn get_file_from_header(headers: axum::http::HeaderMap) -> Result<String> {
    match headers.get("file_id") {
        Some(value) => match value.to_str() {
            Ok(o) => return Ok(o.to_string()),
            Err(_e) => return Err(Error::FileNotFound),
        },
        None => return Err(Error::FileNotFound),
    };
}

async fn build_file_upload(ctx: Ctx, mut multipart: Multipart) -> Result<Json<Value>> {
    let mut tmp_file = tempfile().map_err(|e| {
        error!("Failed to create tempfile: {}", e);
        Error::InternalServerError
    })?;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().ok_or(Error::InternalServerError)?.to_string();
        let data = match field.bytes().await {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to read bytes: {}", e);
                return Err(Error::InternalServerError);
            }
        };

        println!("Length of `{}` is {} bytes", name, data.len());

        // Create a tempfile object from bytes
        match tmp_file.write_all(&data) {
            Ok(()) => {}
            Err(e) => {
                error!("Failed to write to tempfile: {}", e);
                return Err(Error::InternalServerError);
            }
        };
    }

    gcc_container(tmp_file.into()).await.map_err(|e| {
        error!("Failed to build file: {}", e);
        Error::InternalServerError
    })?;

    let json = Json(json!({
        "message": "Successfully uploaded file"
    }));
    Ok(json)
}
