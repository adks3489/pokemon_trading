use sqlx::{Pool, Postgres, postgres::PgQueryResult};
use serde::{Serialize};

pub type DbPool = Pool<Postgres>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i16)]
pub enum Status {
    Pending = 0,
    Filled = 1,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Order {
    pub id: i64,
    pub card_id: i32,
    pub price: i32,
    pub side: i16,
    pub status: i16,
    pub trader_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>
}

pub async fn query_orders(pool: &DbPool, trader_id: i64, limit: Option<i64>) -> sqlx::Result<Vec<Order>> {
    sqlx::query_as!(Order, "SELECT * FROM orders WHERE trader_id = $1 ORDER BY created_at DESC LIMIT $2", trader_id, limit.unwrap_or(50))
        .fetch_all(pool).await
}

pub struct NewOrder {
    pub card_id: i32,
    pub price: i32,
    pub side: i16,
    pub status: i16,
    pub trader_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>
}
pub async fn insert_order(pool: &DbPool, order: NewOrder) -> sqlx::Result<i64>{
    let id = sqlx::query!("INSERT INTO orders (card_id, price, side, status, trader_id, created_at) VALUES ($1, $2, $3, $4, $5, $6) returning id;",
        order.card_id, order.price, order.side, order.status, order.trader_id, order.created_at)
        .fetch_one(pool).await?.id;
    Ok(id)
}

pub async fn update_order_status(pool: &DbPool, order_id: i64, status: Status) -> sqlx::Result<PgQueryResult> {
    sqlx::query!("UPDATE orders SET status = $1 WHERE id = $2", status as i16, order_id)
        .execute(pool).await
}

#[derive(sqlx::FromRow, Serialize)]
pub struct PendingOrder {
    pub id: i64,
    pub side: i16,
    pub price: i32,
    pub card_id: i32,
    // TODO: only id & price is must-have
}
pub async fn query_pending_orders(pool: &DbPool, card_id: i32, side: i16) -> sqlx::Result<Vec<PendingOrder>> {
    sqlx::query_as!(PendingOrder, "SELECT id, side, price, card_id FROM orders WHERE status = 0 AND card_id = $1 AND side = $2", card_id, side)
        .fetch_all(pool).await
}
