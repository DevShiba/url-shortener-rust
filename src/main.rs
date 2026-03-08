mod cache;
mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod routes;
mod utils;

use std::sync::Arc;

use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{
    cache::redis::RedisStore,
    config::Config,
    db::scylla::ScyllaRepository,
    routes::{AppState, build_router},
    utils::base62::Shortener,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = Arc::new(Config::from_env()?);

    tracing::info!("connecting to ScyllaDB, Redis counter, and Redis cache...");

    let (db, cache, shortener) = tokio::try_join!(
        ScyllaRepository::connect(&config.scylla_nodes, &config.scylla_keyspace),
        RedisStore::connect(&config.redis_counter_url, &config.redis_cache_url),
        async { Shortener::new(&config.hashids_salt, config.hashids_min_length) },
    )?;

    let state = Arc::new(AppState {
        db,
        cache,
        shortener,
        config: Arc::clone(&config),
    });

    let router = build_router(state);
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(addr = %addr, "server listening");

    axum::serve(listener, router).await?;

    Ok(())
}