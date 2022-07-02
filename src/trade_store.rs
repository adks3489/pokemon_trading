use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{PgPool};
use anyhow::Result;
use crate::ports::{TradeStore, Trade};

#[derive(Clone)]
pub struct PostgresTradeStoreImpl {
    pub pg_pool: Arc<PgPool>
}

#[async_trait]
impl TradeStore for PostgresTradeStoreImpl {
    async fn insert_trade(&self, card_id: i32, price: i32, buyorder_id: i64, sellorder_id: i64) -> Result<()> {
        sqlx::query!("INSERT INTO trades (card_id, price, buyorder_id, sellorder_id) VALUES ($1, $2, $3, $4)",
        card_id, price, buyorder_id, sellorder_id)
        .execute(&*self.pg_pool).await?;
        Ok(())
    }
    async fn query_trades(&self, card_id: i32, limit: Option<i64>) -> Result<Vec<Trade>> {
        Ok(sqlx::query_as!(Trade, "SELECT * FROM trades WHERE card_id = $1 ORDER BY created_at DESC LIMIT $2", card_id, limit.unwrap_or(50))
        .fetch_all(&*self.pg_pool).await?)
    }
}
