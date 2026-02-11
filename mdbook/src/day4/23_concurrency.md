# Chapter 23: Concurrency -- Thread Pool & Shared State

This chapter adds concurrent job processing to imgforge, applying async/concurrency theory
from Chapter 18 to keep the server responsive during CPU-heavy image transforms.

## 1. The Problem

Image processing is inherently CPU-bound. Operations like grayscale conversion, blur, and
resize involve iterating over every pixel. If we run these directly inside an async handler,
we block the Tokio executor thread -- and while it is blocked, no other requests can be served.

Recall the golden rule from Chapter 18 (Async and Concurrency): **never block the executor**.
Blocking a single worker thread in a 4-thread Tokio runtime means 25% of your server capacity
disappears for the duration of that image transform.

The solution is `tokio::task::spawn_blocking`, which moves the blocking closure onto a
dedicated thread pool that Tokio manages separately from its async worker threads.

## 2. spawn_blocking -- Bridging Async and CPU-Bound Work

Wrap the CPU-intensive `Transform::apply` call so it runs on Tokio's blocking thread pool:

```rust,ignore
let backend = state.backend.clone();
let result = tokio::task::spawn_blocking(move || {
    backend.apply(&image_data, &operation)
}).await
.map_err(|e| Error::InvalidOperation {
    message: format!("Task join error: {e}"),
})??;
```

### Understanding the Double `??`

The return type of `spawn_blocking(...).await` is `Result<T, JoinError>`, where `T` is
whatever the closure returns. In our case the closure returns `Result<Vec<u8>, Error>`, so
the full type is:

```text
Result<Result<Vec<u8>, Error>, JoinError>
```

The first `?` unwraps the outer `JoinError` (task panic or cancellation). The second `?`
unwraps our application-level `Result`. The `map_err` converts the `JoinError` into our
own `Error` type before the first `?` fires.

### Why `move` and Why `Arc`

The `move` keyword transfers ownership of captured variables into the closure. Since the
closure runs on a different thread, all captured values must be `Send + 'static`. The
`backend` field on `AppState` is wrapped in `Arc` so that cloning is cheap -- we get a
new handle to the same backend instance without duplicating its data.

### Comparison with C#

The closest C# equivalent is `Task.Run(() => ...)`, which schedules work on the
`ThreadPool`. The key difference: Rust enforces at compile time that everything captured
by the closure is safe to send across threads (`Send` bound). In C#, thread safety of
captured variables is your responsibility at runtime.

```csharp
// C# equivalent (approximate)
var result = await Task.Run(() => backend.Apply(imageData, operation));
```

## 3. Job Tracking -- Shared State

To let clients check progress of long-running transforms, we add job tracking with shared state.

### Data Model

```rust,ignore
use serde::Serialize;
use uuid::Uuid;

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

### Carrying Jobs in AppState

The jobs map lives in `AppState`, shared across all handlers via Axum's `State`:

```rust,ignore
#[derive(Clone)]
pub struct AppState {
    pub backend: Arc<dyn Transform>,
    pub jobs: Arc<Mutex<HashMap<Uuid, JobStatus>>>,
}
```

Why each wrapper layer:

| Wrapper | Purpose |
|---------|---------|
| `HashMap<Uuid, JobStatus>` | The actual data: job ID to status |
| `Mutex<...>` | Interior mutability -- allows mutation through a shared reference |
| `Arc<...>` | Shared ownership -- multiple tasks hold a handle to the same map |

**Important**: keep the lock scope as small as possible. Lock, update, drop. Never hold
a `Mutex` guard across an `.await` point.

```rust,ignore
// GOOD: lock scope is a single block
{
    let mut jobs = state.jobs.lock().unwrap();
    jobs.insert(job_id, status);
}  // MutexGuard dropped here

// BAD: lock held across await
let mut jobs = state.jobs.lock().unwrap();
some_async_operation().await;  // Other tasks blocked!
jobs.insert(job_id, status);
```

## 4. Job Lifecycle

A transform request follows this lifecycle:

1. **Create** -- insert a `Pending` job before spawning any work
2. **Process** -- update to `Processing` inside the blocking task
3. **Finish** -- update to `Complete` or `Failed` after the transform

### Full Transform Handler with Job Tracking

```rust,ignore
async fn transform_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> std::result::Result<axum::response::Response, Error> {
    // ... extract image_data and operation from multipart ...

    // Step 1: Create job in Pending state
    let job_id = Uuid::new_v4();
    {
        let mut jobs = state.jobs.lock().unwrap();
        jobs.insert(job_id, JobStatus {
            id: job_id,
            state: JobState::Pending,
            error: None,
        });
    }

    // Step 2: Spawn blocking work
    let backend = state.backend.clone();
    let jobs_handle = state.jobs.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Update to Processing
        {
            let mut jobs = jobs_handle.lock().unwrap();
            if let Some(job) = jobs.get_mut(&job_id) {
                job.state = JobState::Processing;
            }
        }

        // Perform the CPU-intensive transform
        let transform_result = backend.apply(&image_data, &operation);

        // Update to Complete or Failed
        {
            let mut jobs_lock = jobs_handle.lock().unwrap();
            if let Some(job) = jobs_lock.get_mut(&job_id) {
                match &transform_result {
                    Ok(_) => job.state = JobState::Complete,
                    Err(e) => {
                        job.state = JobState::Failed;
                        job.error = Some(e.to_string());
                    }
                }
            }
        }

        transform_result
    })
    .await
    .map_err(|e| Error::InvalidOperation {
        message: format!("Task join error: {e}"),
    })??;

    // Step 3: Return result with job ID in header
    use axum::response::IntoResponse;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "image/png")],
        [(axum::http::header::HeaderName::from_static("x-job-id"), job_id.to_string())],
        result,
    ).into_response())
}
```

Notice how each lock block acquires, updates, and drops immediately. The lock is never held
during image processing.

## 5. Job Status Endpoint

Clients can query job progress via `GET /jobs/{id}`:

```rust,ignore
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
```

The Axum `State` extractor requires the state type to implement `Clone`. Because `AppState`
contains only `Arc` fields, cloning is cheap (reference count increment, no data copy).

The job ID is returned via the `X-Job-Id` response header. The client can then poll:

```bash
# Submit a transform and capture the job ID header
curl -v -X POST -F "image=@photo.jpg" -F "operation=grayscale" \
    http://localhost:3000/transform 2>&1 | grep -i x-job-id

# Query status with the returned ID
curl http://localhost:3000/jobs/<job-id>
```

Register the route alongside the existing ones:

```rust,ignore
let app = Router::new()
    .route("/", get(index_handler))
    .route("/health", get(health_handler))
    .route("/transform", post(transform_handler))
    .route("/jobs/{id}", get(job_status_handler))
    .with_state(state);
```

## 6. Observing Concurrency

Fire multiple requests simultaneously to see concurrent processing:

```bash
# Terminal 1
curl -X POST -F "image=@photo.jpg" -F "operation=grayscale" http://localhost:3000/transform &
# Terminal 2
curl -X POST -F "image=@photo.jpg" -F "operation=blur" http://localhost:3000/transform &
# Terminal 3: health check responds instantly even during heavy transforms
curl http://localhost:3000/health
```

Both transforms run simultaneously on Tokio's blocking thread pool (grows on demand, up to
512 threads), separate from async workers. The `/health` endpoint stays responsive.

## 7. Arc\<Mutex\<T\>\> vs Alternatives

| Approach | When to use | Trade-off |
|----------|------------|-----------|
| `Arc<Mutex<T>>` | Simple shared state | Blocking lock, fine for small critical sections |
| `tokio::sync::Mutex` | Lock held across `.await` | Async-aware, but slower for sync-only access |
| `dashmap` | High-contention concurrent map | Sharded locks, less contention |
| `RwLock` | Read-heavy workloads | Multiple readers, single writer |

For imgforge's job map, `Arc<Mutex<HashMap>>` is the right choice -- lock duration is
microseconds, we never hold across `.await`, and contention is low. The Tokio documentation
itself recommends `std::sync::Mutex` when the critical section is short and does not span
an await.

## 8. Exercise

The exercise provides handlers with `todo!()` markers. Implement the concurrent pipeline.

### Task 1: Job Creation

Insert a `Pending` job into the shared jobs map:

```rust,ignore
// TODO: Create a new job ID and insert a Pending job into state.jobs
let job_id = todo!();
```

### Task 2: spawn_blocking with State Updates

Wrap the transform call in `spawn_blocking`, updating the job state at each transition:

```rust,ignore
// TODO: Clone the necessary handles, spawn_blocking, and update job state
// through the Pending -> Processing -> Complete/Failed lifecycle
let result = todo!();
```

### Task 3: Job Status Endpoint

Implement the handler that looks up a job by ID and returns its status as JSON:

```rust,ignore
async fn job_status_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> std::result::Result<Json<JobStatus>, Error> {
    // TODO: Lock the jobs map, find the job, return its status
    todo!()
}
```

### Testing Your Implementation

```bash
# Start the server
cargo run

# Submit a transform and capture the job ID
curl -v -X POST -F "image=@test.png" -F "operation=grayscale" \
    http://localhost:3000/transform 2>&1 | grep -i x-job-id

# Query the job status (replace with actual ID)
curl http://localhost:3000/jobs/<job-id>
```

Expected responses when querying quickly: `{"state":"processing",...}` then `{"state":"complete",...}` (lowercase due to `#[serde(rename_all = "lowercase")]`).

### Bonus: List All Jobs

Add a `GET /jobs` endpoint that returns all tracked jobs:

```rust,ignore
async fn list_jobs_handler(
    State(state): State<AppState>,
) -> Json<Vec<JobStatus>> {
    let jobs = state.jobs.lock().unwrap();
    Json(jobs.values().cloned().collect())
}
```

Register it in the router:

```rust,ignore
.route("/jobs", get(list_jobs_handler))
```

## Summary

- **`spawn_blocking`** moves CPU-bound transforms off the async executor, keeping the server
  responsive.
- **`Arc<Mutex<HashMap>>`** provides safe shared state for job tracking. `Arc` for shared
  ownership, `Mutex` for interior mutability.
- **Short lock scopes** are critical -- lock, update, drop. Never hold across `.await`.
- **Job lifecycle** (Pending -> Processing -> Complete/Failed) gives clients visibility into
  long-running operations via a status endpoint.
- The **double `??`** pattern handles both `JoinError` and application errors in one chain.
