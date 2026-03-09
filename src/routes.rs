use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    routing::{get, post},
};
use ::governor::clock::QuantaInstant;
use ::governor::middleware::NoOpMiddleware;
use tower_governor::{GovernorLayer, governor::GovernorConfig, key_extractor::PeerIpKeyExtractor};
use tower_http::{
    compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer,
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

pub fn build_router(
    state: Arc<AppState>,
    rate_limit: Arc<GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>>>,
) -> Router {
    let shorten_router = Router::new()
        .route("/shorten", post(shorten_handler))
        .layer(GovernorLayer::new(rate_limit));

    Router::new()
        .merge(shorten_router)
        .route("/{shortcode}", get(redirect_handler))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
