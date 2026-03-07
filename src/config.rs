use crate::errors::AppError;

#[derive(Debug)]
pub struct Config {
    pub server_port: u16,
    pub short_domain: String,
    /// Parsed from a comma-separated string: "node1:9042,node2:9042"
    pub scylla_nodes: Vec<String>,
    pub scylla_keyspace: String,
    pub redis_counter_url: String,
    pub redis_cache_url: String,
    pub hashids_salt: String,
    pub hashids_min_length: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenv::dotenv().ok();

        Ok(Self {
            server_port: env_var("SERVER_PORT")?
                .parse::<u16>()
                .map_err(|_| AppError::Config(
                    "SERVER_PORT must be a valid port number (1–65535)".into(),
                ))?,

            short_domain: env_var("SHORT_DOMAIN")?,

            scylla_nodes: env_var("SCYLLA_NODES")?
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect(),

            scylla_keyspace: env_var("SCYLLA_KEYSPACE")?,

            redis_counter_url: env_var("REDIS_COUNTER_URL")?,

            redis_cache_url: env_var("REDIS_CACHE_URL")?,

            hashids_salt: env_var("HASHIDS_SALT")?,

            hashids_min_length: env_var("HASHIDS_MIN_LENGTH")?
                .parse::<usize>()
                .map_err(|_| AppError::Config(
                    "HASHIDS_MIN_LENGTH must be a positive integer".into(),
                ))?,
        })
    }
}

fn env_var(key: &str) -> Result<String, AppError> {
    std::env::var(key)
        .map_err(|_| AppError::Config(format!("missing environment variable: {key}")))
}
