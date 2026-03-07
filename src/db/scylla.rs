use std::sync::Arc;

use scylla::{
    client::session::Session, client::session_builder::SessionBuilder,
    statement::prepared::PreparedStatement,
};

use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct ScyllaRepository {
    session: Arc<Session>,
    insert_stmt: Arc<PreparedStatement>,
    select_stmt: Arc<PreparedStatement>,
}

impl ScyllaRepository {
    pub async fn connect(nodes: &[String], keyspace: &str) -> Result<Self, AppError> {
        let session = SessionBuilder::new()
            .known_nodes(nodes)
            .use_keyspace(keyspace, false)
            .build()
            .await
            .map_err(|e| AppError::Database(format!("failed to connect to ScyllaDB: {e}")))?;

        let insert_stmt = session
            .prepare(
                "INSERT INTO url (shortcode, long_url, created_at) \
                 VALUES (?, ?, toTimestamp(now())) IF NOT EXISTS",
            )
            .await
            .map_err(|e| AppError::Database(format!("failed to prepare insert statement: {e}")))?;

        let select_stmt = session
            .prepare("SELECT long_url FROM url WHERE shortcode = ?")
            .await
            .map_err(|e| AppError::Database(format!("failed to prepare select statement: {e}")))?;

        Ok(Self {
            session: Arc::new(session),
            insert_stmt: Arc::new(insert_stmt),
            select_stmt: Arc::new(select_stmt),
        })
    }

    pub async fn insert_url(&self, shortcode: &str, long_url: &str) -> Result<(), AppError> {
        self.session
            .execute_unpaged(&*self.insert_stmt, (shortcode, long_url))
            .await
            .map_err(|e| AppError::Database(format!("insert failed: {e}")))?;
        Ok(())
    }

    pub async fn get_url(&self, shortcode: &str) -> Result<Option<String>, AppError> {
        let result = self
            .session
            .execute_unpaged(&*self.select_stmt, (shortcode,))
            .await
            .map_err(|e| AppError::Database(format!("select failed: {e}")))?;

        let maybe_row = result
            .into_rows_result()
            .map_err(|e| AppError::Database(format!("failed to read rows: {e}")))?
            .maybe_first_row::<(String,)>()
            .map_err(|e| AppError::Database(format!("failed to deserialize row: {e}")))?;

        Ok(maybe_row.map(|(long_url,)| long_url))
    }
}
