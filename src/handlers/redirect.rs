use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};

use crate::{errors::AppError, routes::AppState};

#[tracing::instrument(skip(state), fields(shortcode = %shortcode))]
pub async fn redirect_handler(
    State(state): State<Arc<AppState>>,
    Path(shortcode): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let long_url = match state.cache.get_url(&shortcode).await? {
        Some(url) => {
            tracing::debug!("cache hit");
            url
        }
        None => {
            tracing::debug!("cache miss, querying database");
            let url = state
                .db
                .get_url(&shortcode)
                .await?
                .ok_or_else(|| AppError::NotFound(shortcode.clone()))?;

            state.cache.set_url(&shortcode, &url).await?;
            url
        }
    };

    let location = HeaderValue::from_str(&long_url).map_err(|_| {
        AppError::InvalidUrl(format!(
            "stored URL '{long_url}' is not a valid header value"
        ))
    })?;

    Ok((
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, location)],
    ))
}
