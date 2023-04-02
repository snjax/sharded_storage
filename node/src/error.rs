use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

pub type AppResult<T> = Result<T, AppError>;

pub struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(inner: anyhow::Error) -> Self {
        AppError(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = {
            tracing::debug!("stacktrace: {}", self.0.backtrace());
            (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
