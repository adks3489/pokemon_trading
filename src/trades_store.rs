use sqlx::{Pool, Postgres, postgres::PgQueryResult};
use serde::{Serialize};

pub type DbPool = Pool<Postgres>;

pub async fn insert_trade(pool: &DbPool, card_id: i32, price: i32, buyorder_id: i64, sellorder_id: i64) -> sqlx::Result<PgQueryResult> {
    sqlx::query!("INSERT INTO trades (card_id, price, buyorder_id, sellorder_id) VALUES ($1, $2, $3, $4)",
        card_id, price, buyorder_id, sellorder_id)
        .execute(pool).await
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Trade {
    pub id: i64,
    pub card_id: i32,
    pub price: i32,
    pub buyorder_id: i64,
    pub sellorder_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>
}

pub async fn query_trades(pool: &DbPool, card_id: i32, limit: Option<i64>) -> sqlx::Result<Vec<Trade>> {
    sqlx::query_as!(Trade, "SELECT * FROM trades WHERE card_id = $1 ORDER BY created_at DESC LIMIT $2", card_id, limit.unwrap_or(50))
        .fetch_all(pool).await
}
