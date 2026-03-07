use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("shortcode not found: {0}")]
    NotFound(String),

    #[error("failed to encode shortcode")]
    Encoding,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self);

        let (status, message) = match &self {
            AppError::Config(_) | AppError::Encoding => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
            ),
            AppError::Database(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database unavailable".to_owned(),
            ),
            AppError::Cache(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "cache unavailable".to_owned(),
            ),
            AppError::InvalidUrl(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::NotFound(code) => (
                StatusCode::NOT_FOUND,
                format!("shortcode '{code}' not found"),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
