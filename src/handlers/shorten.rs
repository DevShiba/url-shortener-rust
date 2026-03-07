use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use url::Url;

use crate::{
    errors::AppError,
    models::{ShortenRequest, ShortenResponse},
    routes::AppState,
};

#[tracing::instrument(skip(state), fields(long_url = %req.long_url))]
pub async fn shorten_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ShortenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let parsed = Url::parse(&req.long_url)
        .map_err(|_| AppError::InvalidUrl(format!("'{}' is not a valid URL", req.long_url)))?;

    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(AppError::InvalidUrl(format!(
                "scheme '{}' is not allowed; only http and https are accepted",
                scheme
            )));
        }
    }

    let id = state.cache.next_id().await?;
    let shortcode = state.shortener.encode(id)?;

    state.db.insert_url(&shortcode, &req.long_url).await?;
    state.cache.set_url(&shortcode, &req.long_url).await?;

    tracing::info!(shortcode = %shortcode, "URL shortened successfully");

    Ok((StatusCode::CREATED, Json(ShortenResponse { shortcode })))
}
