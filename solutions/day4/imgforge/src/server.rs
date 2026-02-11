use axum::extract::{Multipart, Path, State};
use axum::response::{Html, Json};
use axum::routing::{get, post};
use axum::Router;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::error::Error;
use crate::transform::{Operation, Transform};

// -- App State

#[derive(Clone)]
pub struct AppState {
    pub backend: Arc<dyn Transform>,
    pub jobs: Arc<Mutex<HashMap<Uuid, JobStatus>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobStatus {
    pub id: Uuid,
    pub state: JobState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Pending,
    Processing,
    Complete,
    Failed,
}

// -- Router

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/transform", post(transform_handler))
        .route("/jobs/{id}", get(job_status_handler))
        .with_state(state)
}

// -- Handlers

async fn index_handler() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<body>
    <h1>imgforge</h1>
    <form action="/transform" method="post" enctype="multipart/form-data">
        <label>Image: <input type="file" name="image" accept="image/*"></label><br>
        <label>Operation:
            <select name="operation">
                <option value="grayscale">Grayscale</option>
                <option value="blur">Blur</option>
                <option value="resize">Resize</option>
            </select>
        </label><br>
        <label>Width: <input type="number" name="width" value="800"></label>
        <label>Height: <input type="number" name="height" value="600"></label>
        <label>Sigma: <input type="number" name="sigma" value="3.0" step="0.1"></label><br>
        <button type="submit">Transform</button>
    </form>
</body>
</html>"#,
    )
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn transform_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> std::result::Result<axum::response::Response, Error> {
    let mut image_data: Option<Vec<u8>> = None;
    let mut operation_str: Option<String> = None;
    let mut width: u32 = 800;
    let mut height: u32 = 600;
    let mut sigma: f32 = 3.0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::InvalidOperation {
            message: format!("Multipart error: {e}"),
        })?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "image" => {
                image_data = Some(field.bytes().await.map_err(|e| Error::InvalidOperation {
                    message: format!("Failed to read image: {e}"),
                })?.to_vec());
            }
            "operation" => {
                operation_str = Some(field.text().await.map_err(|e| Error::InvalidOperation {
                    message: format!("Failed to read operation: {e}"),
                })?);
            }
            "width" => {
                if let Ok(text) = field.text().await {
                    width = text.parse().unwrap_or(800);
                }
            }
            "height" => {
                if let Ok(text) = field.text().await {
                    height = text.parse().unwrap_or(600);
                }
            }
            "sigma" => {
                if let Ok(text) = field.text().await {
                    sigma = text.parse().unwrap_or(3.0);
                }
            }
            _ => {}
        }
    }

    let image_data = image_data.ok_or_else(|| Error::InvalidOperation {
        message: "No image field in multipart".to_string(),
    })?;
    let operation_str = operation_str.unwrap_or_else(|| "grayscale".to_string());

    let operation = match operation_str.as_str() {
        "resize" => Operation::Resize { width, height },
        "grayscale" => Operation::Grayscale,
        "blur" => Operation::Blur { sigma },
        other => {
            return Err(Error::InvalidOperation {
                message: format!("Unknown operation: {other}"),
            });
        }
    };

    // Create a job for tracking
    let job_id = Uuid::new_v4();
    {
        let mut jobs = state.jobs.lock().unwrap();
        jobs.insert(
            job_id,
            JobStatus {
                id: job_id,
                state: JobState::Pending,
                error: None,
            },
        );
    }

    // Move CPU-bound work to a blocking task
    let backend = state.backend.clone();
    let jobs = state.jobs.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Mark as processing
        {
            let mut jobs = jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(&job_id) {
                job.state = JobState::Processing;
            }
        }

        let result = backend.apply(&image_data, &operation);

        // Update job status
        {
            let mut jobs_lock = jobs.lock().unwrap();
            if let Some(job) = jobs_lock.get_mut(&job_id) {
                match &result {
                    Ok(_) => job.state = JobState::Complete,
                    Err(e) => {
                        job.state = JobState::Failed;
                        job.error = Some(e.to_string());
                    }
                }
            }
        }

        result
    })
    .await
    .map_err(|e| Error::InvalidOperation {
        message: format!("Task join error: {e}"),
    })??;

    // Return the processed image
    Ok((
        [(axum::http::header::CONTENT_TYPE, "image/png")],
        [(axum::http::header::HeaderName::from_static("x-job-id"), job_id.to_string())],
        result,
    )
        .into_response())
}

async fn job_status_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<JobStatus>, Error> {
    let jobs = state.jobs.lock().unwrap();
    let status = jobs.get(&id).cloned().ok_or_else(|| Error::InvalidOperation {
        message: format!("Job {id} not found"),
    })?;
    Ok(Json(status))
}

use axum::response::IntoResponse;
