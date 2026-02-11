use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // -- Domain errors (no #[from], constructed explicitly)
    UnsupportedFormat { format: String },
    DimensionTooLarge { width: u32, height: u32 },
    InvalidOperation { message: String },

    // -- External errors (auto-converted via #[from])
    #[from]
    Io(std::io::Error),
    #[from]
    Image(image::ImageError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnsupportedFormat { format } => write!(f, "Unsupported format: {format}"),
            Error::DimensionTooLarge { width, height } => {
                write!(f, "Dimensions too large: {width}x{height}")
            }
            Error::InvalidOperation { message } => write!(f, "Invalid operation: {message}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Image(e) => write!(f, "Image error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

// Enable Axum to convert our Error into an HTTP response
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

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
