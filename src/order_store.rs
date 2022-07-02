
use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{PgPool};
use anyhow::Result;
use crate::ports::{OrderStore, Order, NewOrder, PendingOrder, Status};

#[derive(Clone)]
pub struct PostgresOrderStoreImpl {
    pub pg_pool: Arc<PgPool>
}

#[async_trait]
impl OrderStore for PostgresOrderStoreImpl {
    async fn query_orders(&self, trader_id: i64, limit: Option<i64>) -> Result<Vec<Order>> {
        Ok(sqlx::query_as!(Order, "SELECT * FROM orders WHERE trader_id = $1 ORDER BY created_at DESC LIMIT $2", trader_id, limit.unwrap_or(50))
            .fetch_all(&*self.pg_pool).await?)
    }
    async fn insert_order(&self, order: NewOrder) -> Result<i64> {
        let id = sqlx::query!("INSERT INTO orders (card_id, price, side, status, trader_id, created_at) VALUES ($1, $2, $3, $4, $5, $6) returning id;",
            order.card_id, order.price, order.action as i16, order.status, order.trader_id, order.created_at)
            .fetch_one(&*self.pg_pool).await?.id;
        Ok(id)
    }
    async fn update_order_status(&self, order_id: i64, status: Status) -> Result<()> {
        sqlx::query!("UPDATE orders SET status = $1 WHERE id = $2", status as i16, order_id)
            .execute(&*self.pg_pool).await?;
        Ok(())
    }
    async fn query_pending_orders(&self, card_id: i32, side: i16) -> Result<Vec<PendingOrder>> {
        Ok(sqlx::query_as!(PendingOrder, "SELECT id, side, price, card_id FROM orders WHERE status = 0 AND card_id = $1 AND side = $2", card_id, side)
            .fetch_all(&*self.pg_pool).await?)
    }
}
