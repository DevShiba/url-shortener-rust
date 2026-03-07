use fred::prelude::*;

use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct RedisStore {
    counter: Client,
    cache: Client,
}

impl RedisStore {
    pub async fn connect(counter_url: &str, cache_url: &str) -> Result<Self, AppError> {
        let counter = build_client(counter_url).await?;
        let cache = build_client(cache_url).await?;
        Ok(Self { counter, cache })
    }

    pub async fn next_id(&self) -> Result<u64, AppError> {
        self.counter
            .incr::<u64, _>("url:counter")
            .await
            .map_err(|e| AppError::Cache(format!("INCR failed: {e}")))
    }

    pub async fn get_url(&self, shortcode: &str) -> Result<Option<String>, AppError> {
        self.cache
            .get::<Option<String>, _>(shortcode)
            .await
            .map_err(|e| AppError::Cache(format!("GET failed: {e}")))
    }

    pub async fn set_url(&self, shortcode: &str, long_url: &str) -> Result<(), AppError> {
        self.cache
            .set::<(), _, _>(
                shortcode,
                long_url,
                Some(Expiration::EX(86_400)),
                None,
                false,
            )
            .await
            .map_err(|e| AppError::Cache(format!("SET failed: {e}")))
    }
}

async fn build_client(url: &str) -> Result<Client, AppError> {
    let config = Config::from_url(url)
        .map_err(|e| AppError::Cache(format!("invalid Redis URL '{url}': {e}")))?;

    let client = Builder::from_config(config)
        .build()
        .map_err(|e| AppError::Cache(format!("failed to build Redis client: {e}")))?;

    client
        .init()
        .await
        .map_err(|e| AppError::Cache(format!("failed to connect to Redis at '{url}': {e}")))?;

    Ok(client)
}
