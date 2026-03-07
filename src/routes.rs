use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    cache::redis::RedisStore,
    config::Config,
    db::scylla::ScyllaRepository,
    handlers::{redirect::redirect_handler, shorten::shorten_handler},
    utils::base62::Shortener,
};

#[derive(Debug)]
pub struct AppState {
    pub db: ScyllaRepository,
    pub cache: RedisStore,
    pub shortener: Shortener,
    pub config: Arc<Config>,
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/shorten", post(shorten_handler))
        .route("/{shortcode}", get(redirect_handler))
        .with_state(state)
}
