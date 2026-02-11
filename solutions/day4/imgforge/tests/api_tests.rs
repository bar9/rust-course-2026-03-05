use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use imgforge::server::{AppState, JobState};
use imgforge::transform;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

fn test_state() -> AppState {
    AppState {
        backend: Arc::from(transform::default_backend()),
        jobs: Arc::new(Mutex::new(HashMap::new())),
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = imgforge::server::router(test_state());

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_index_returns_html() {
    let app = imgforge::server::router(test_state());

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("imgforge"));
    assert!(html.contains("<form"));
}

#[tokio::test]
async fn test_job_not_found() {
    let app = imgforge::server::router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/jobs/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_job_status_lookup() {
    let state = test_state();
    let job_id = uuid::Uuid::new_v4();

    // Insert a job manually
    {
        let mut jobs = state.jobs.lock().unwrap();
        jobs.insert(
            job_id,
            imgforge::server::JobStatus {
                id: job_id,
                state: JobState::Complete,
                error: None,
            },
        );
    }

    let app = imgforge::server::router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["state"], "complete");
}
