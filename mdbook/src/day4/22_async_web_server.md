# Chapter 22: Async Web Server -- Axum HTTP API

In previous chapters, imgforge worked as a CLI tool. This chapter adds a second mode -- an async web server built with Axum -- while keeping the CLI intact. The library core is shared between both modes.

This chapter assumes familiarity with `async`/`await` and Tokio from [Chapter 18: Async and Concurrency](../day3/18_async_concurrency.md).

## 1. From CLI to Server

The transform logic does not change. We add a new *interface* on top of the same library:

```text
imgforge cli resize --width 800 input.jpg output.jpg   # existing CLI mode
imgforge serve --port 3000                              # new server mode
```

Both commands call into the same `Transform` trait and error types. The only new code is the HTTP layer.

### New Dependencies

```toml
[dependencies]
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
```

Axum is built on [Tower](https://docs.rs/tower) (middleware framework) and [Tokio](https://tokio.rs) (async runtime). There are no hidden macros or code generation steps -- handlers are plain async functions.

## 2. Axum Basics

Axum has three core concepts:

- **Router** -- maps HTTP methods and paths to handler functions.
- **Handlers** -- plain `async fn` functions. No controller classes, no attributes.
- **Extractors** -- typed parameters in the handler signature that Axum fills in from the request.

### Comparison with ASP.NET

| ASP.NET / C# | Axum | Notes |
|---|---|---|
| `[ApiController]` + `[Route("api/...")]` | `Router::new().route("/api/...", get(handler))` | Routing is explicit code, not attributes |
| Action method on a Controller | `async fn handler(...) -> impl IntoResponse` | Plain async functions, no base class |
| `[FromBody]`, `[FromQuery]`, `[FromRoute]` | `Json<T>`, `Query<T>`, `Path<T>` extractors | Type-driven extraction from function signature |
| Middleware pipeline (`app.UseXxx()`) | Tower layers | Composable, reusable middleware |
| DI container (`IServiceProvider`) | `State<T>` extractor | Explicit state passing, no service locator |
| `IActionResult` / `ActionResult<T>` | `impl IntoResponse` | Any type implementing `IntoResponse` works |
| Model validation (`[Required]`) | Manual validation or `validator` crate | No built-in attribute-based validation |
| `IOptions<T>` / config binding | `State<AppConfig>` | Configuration is data passed as state |

The key insight: Axum handlers are **just async functions**. No framework magic, no hidden base class, no DI container. You pass state explicitly, return types that implement `IntoResponse`, and the compiler checks everything at build time.

## 3. Server Module -- server.rs

### Application State

The server needs shared state for the transform backend and a job tracker:

```rust,ignore
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde::Serialize;
use crate::transform::{Operation, Transform};
use crate::error::Error;

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
```

The `backend` is `Arc<dyn Transform>` -- the same trait used by the CLI path. `JobStatus` is a struct with a `JobState` enum -- the struct carries the job ID and optional error message, while the enum represents the lifecycle states. The `#[serde(rename_all = "lowercase")]` attribute serializes variants as `"pending"`, `"complete"`, etc. in JSON responses.

### The Router

```rust,ignore
use axum::{Router, routing::{get, post}};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        .route("/transform", post(transform_handler))
        .with_state(state)
}
```

Each `.route()` binds an HTTP method and path to a handler. The `.with_state(state)` call makes `AppState` available via the `State` extractor. Compare with ASP.NET minimal APIs (`MapGet`/`MapPost`) -- similar structure, but Axum resolves parameters at compile time.

## 4. Simple Handlers

### Index -- HTML Upload Form

```rust,ignore
use axum::response::Html;

pub async fn index_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html><body>
    <h1>imgforge</h1>
    <form action="/transform" method="post" enctype="multipart/form-data">
        <input type="file" name="image" accept="image/*" required><br>
        <select name="operation">
            <option value="resize">Resize</option>
            <option value="grayscale">Grayscale</option>
            <option value="blur">Blur</option>
        </select><br>
        Width: <input type="number" name="width" value="800">
        Height: <input type="number" name="height" value="600"><br>
        <button type="submit">Transform</button>
    </form>
</body></html>"#)
}
```

`Html<&'static str>` implements `IntoResponse`. Axum sets `Content-Type: text/html` automatically.

### Health Check -- JSON Response

```rust,ignore
use axum::Json;
use serde_json::{json, Value};

pub async fn health_handler() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
```

`Json<T>` serializes the value and sets `Content-Type: application/json`.

### How IntoResponse Works

Both handlers return different types, yet both are valid because they implement `IntoResponse` -- Axum's key response abstraction. Built-in implementations include:

- `String` / `&str` -- plain text
- `Html<T>` -- HTML with correct content type
- `Json<T>` -- JSON with correct content type
- `(StatusCode, T)` -- tuple with explicit status code
- `Response` -- a full HTTP response you build yourself

## 5. Multipart Upload Handler

The transform handler accepts a multipart form upload, extracts the image and parameters, runs the transform, and returns the processed image:

```rust,ignore
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn transform_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> std::result::Result<axum::response::Response, Error> {
    let mut image_data: Option<Vec<u8>> = None;
    let mut operation_str: Option<String> = None;
    let mut width: u32 = 800;
    let mut height: u32 = 600;
    let mut sigma: f32 = 3.0;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| Error::InvalidOperation {
            message: format!("Multipart error: {e}"),
        })?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "image" => {
                image_data = Some(field.bytes().await
                    .map_err(|e| Error::InvalidOperation {
                        message: format!("Failed to read image: {e}"),
                    })?.to_vec());
            }
            "operation" => {
                operation_str = Some(field.text().await
                    .map_err(|e| Error::InvalidOperation {
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

    // Convert the string operation to the Operation enum
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

    // Call the same Transform trait used by the CLI
    let result = state.backend.apply(&image_data, &operation)?;

    Ok((
        [(axum::http::header::CONTENT_TYPE, "image/png")],
        result,
    ).into_response())
}
```

Key points:

- **`State(state): State<AppState>`** -- Axum clones the state for each request, which is why `AppState` derives `Clone` and wraps data in `Arc`.
- **`mut multipart: Multipart`** -- parses `multipart/form-data`. Fields are consumed as an async stream.
- **String → `Operation` enum conversion** -- the form sends operation names as strings; we convert them to the typed `Operation` enum before calling the backend.
- **`std::result::Result<Response, Error>`** -- uses the standard `Result` (not our alias) because the return type is a concrete `Response`, not a generic `T`. The `?` operator propagates errors, which requires `Error` to implement `IntoResponse` (next section).
- **Return tuple with `.into_response()`** -- Axum implements `IntoResponse` for tuples of headers and body, a concise way to build responses.

The `Transform::apply` call is reused from the CLI path. No duplication, no adapter layer.

## 6. JSON Error Responses

For `?` to work in handlers, our `Error` type must implement `IntoResponse`:

```rust,ignore
use axum::{http::StatusCode, response::IntoResponse};

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Error::UnsupportedFormat { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::DimensionTooLarge { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::InvalidOperation { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::Image(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
        };
        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}
```

Once implemented, every `?` in a handler that hits an error automatically produces a JSON response with the correct HTTP status code:

- Client errors (`UnsupportedFormat`, `DimensionTooLarge`, `InvalidOperation`) return `400 Bad Request`
- Image processing failures return `422 Unprocessable Entity`
- Internal errors return `500 Internal Server Error`
- The body is always `{"error": "description"}`

Compare with ASP.NET's `ProblemDetails` middleware or custom exception filters. The Rust approach is more explicit -- you define the mapping, and the compiler ensures every variant is handled (no wildcard `_` catch-all needed).

## 7. Dual-Mode main.rs

### The Serve Subcommand

The `Serve` variant was already declared as a placeholder in Ch20's `config.rs`. No changes are needed to the CLI configuration -- `Command::Serve { port }` is ready to use.

### The run_server Function in lib.rs

The server function creates a Tokio runtime, builds the application state, and starts Axum:

```rust,ignore
fn run_server(port: u16) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let state = server::AppState {
            backend: std::sync::Arc::from(transform::default_backend()),
            jobs: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        };

        let app = server::router(state);
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(|e| Error::Io(e))?;

        println!("imgforge server listening on http://localhost:{port}");
        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Io(e))?;

        Ok(())
    })
}
```

Note: `run_server` is a **synchronous** function that creates and blocks on a Tokio runtime internally. This avoids requiring `#[tokio::main]` on `main()` and keeps the `run()` dispatcher simple -- it works for both sync (CLI) and async (server) modes.

### Thin main.rs

```rust,ignore
use clap::Parser;

fn main() -> imgforge::Result<()> {
    let config = imgforge::Config::parse();
    imgforge::run(config)
}
```

The `run()` function in `lib.rs` dispatches to `run_cli()` or `run_server()` based on the `Command` variant. The main binary stays at three lines.

Same binary, two modes. The library core (`Transform` trait, error types, image processing) is shared. This is the library-first pattern: `main.rs` is thin, both interfaces are entry points into the same library.

## 8. Testing the Server

Test manually with `curl`:

```bash
# Start the server
cargo run -- serve --port 3000

# Health check
curl http://localhost:3000/health
# Expected: {"status":"ok"}

# Upload and transform
curl -X POST http://localhost:3000/transform \
  -F "image=@test_input.jpg" \
  -F "operation=resize" \
  -F "width=400" \
  --output result.jpg
```

For automated testing, use Tower's `oneshot` to send requests without a real TCP listener:

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn health_returns_ok() {
        let state = AppState {
            backend: Arc::from(crate::transform::default_backend()),
            jobs: Arc::new(Mutex::new(HashMap::new())),
        };
        let app = router(state);

        let response = app
            .oneshot(Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

No network involved -- the test runs in memory. The test uses `default_backend()` for simplicity; in [Chapter 24: Testing](./24_testing_polish.md) we will explore using mock backends for more targeted tests.

## 9. Exercise

The starter code has `todo!()` markers in three locations.

### Task 1: Router Setup

In `server.rs`, add the `/transform` route:

```rust,ignore
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/health", get(health_handler))
        // TODO: add a POST route for "/transform" that calls transform_handler
        .with_state(state)
}
```

### Task 2: Transform Handler

Implement multipart field extraction:

```rust,ignore
pub async fn transform_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> std::result::Result<axum::response::Response, Error> {
    let mut image_data: Option<Vec<u8>> = None;
    let mut operation_str: Option<String> = None;
    let mut width: u32 = 800;
    let mut height: u32 = 600;
    let mut sigma: f32 = 3.0;

    while let Some(field) = multipart.next_field().await
        .map_err(|_| Error::InvalidOperation {
            message: "Multipart read error".into(),
        })?
    {
        let name = field.name().unwrap_or("").to_string();
        // TODO: match on name and extract:
        //   "image"     -> read bytes into image_data
        //   "operation" -> read text into operation_str
        //   "width"     -> parse into width
        //   "height"    -> parse into height
        //   "sigma"     -> parse into sigma
        todo!("Extract multipart fields");
    }

    let image_data = image_data.ok_or_else(|| Error::InvalidOperation {
        message: "No image field in multipart".into(),
    })?;
    // TODO: convert operation_str to an Operation enum, then call:
    let result = state.backend.apply(&image_data, &operation)?;
    Ok(([(axum::http::header::CONTENT_TYPE, "image/png")], result).into_response())
}
```

### Task 3: Error-to-Response Mapping

In `error.rs`, implement `IntoResponse` for `Error`:

```rust,ignore
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO: match on each variant and map to HTTP status code:
        //   UnsupportedFormat  -> 400 BAD_REQUEST
        //   DimensionTooLarge  -> 400 BAD_REQUEST
        //   InvalidOperation   -> 400 BAD_REQUEST
        //   Io                 -> 500 INTERNAL_SERVER_ERROR
        //   Image              -> 422 UNPROCESSABLE_ENTITY
        // Return JSON body: {"error": "<message>"}
        todo!("Map error variants to status codes and JSON body");
    }
}
```

### Verification

1. `cargo build` -- should compile without errors.
2. `cargo run -- serve --port 3000` -- start the server.
3. `curl http://localhost:3000/health` -- verify JSON response.
4. Open `http://localhost:3000` in a browser -- verify the upload form.

### Bonus: GET /info Endpoint

Add a `GET /info` endpoint returning the backend name as JSON:

```json
{"backend": "image-rs", "version": "0.1.0"}
```

This requires adding an `info_handler` function, the route to the router, and optionally a `name()` method on the `Transform` trait.

## Summary

| Concept | What we covered |
|---|---|
| Axum fundamentals | Router, handlers, extractors, `IntoResponse` |
| Shared state | `AppState` with `Arc<dyn Transform>` and `Arc<Mutex<...>>` |
| Multipart uploads | Extracting file data and form fields from HTTP requests |
| Error handling | `impl IntoResponse for Error` maps domain errors to HTTP status codes |
| Dual-mode binary | Same crate exposes both CLI and server interfaces |
| Testing | In-memory request testing with Tower's `oneshot` |

The server module added roughly 150 lines to imgforge. The transform logic, error types, and backend implementations were not modified. This is the payoff of library-first architecture: new interfaces are cheap to add when the core is well-separated.

Next up: [Chapter 23](./23_concurrency.md) adds a thread pool for CPU-bound image processing, with a `/jobs/{id}` status endpoint for long-running transforms.
