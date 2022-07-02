use std::sync::Arc;
use sqlx::{PgPool};
use log::{error};
use async_trait::async_trait;
use crate::ports::TraderStore;

#[derive(Clone)]
pub struct PostgresTraderStoreImpl {
    pub pg_pool: Arc<PgPool>
}

#[async_trait]
impl TraderStore for PostgresTraderStoreImpl {
    async fn is_exist(&self, id: i64) -> Option<bool> {
        let r = sqlx::query!("SELECT EXISTS (SELECT 1 FROM traders WHERE id = $1 LIMIT 1)", id)
            .fetch_one(&*self.pg_pool).await;
        match r {
            Ok(r) => r.exists,
            Err(e) => {
                error!("Failed to query trader: {}", e);
                None
            },
        }
    }
}
