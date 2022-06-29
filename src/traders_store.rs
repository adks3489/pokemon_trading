use sqlx::{Pool, Postgres};

pub type DbPool = Pool<Postgres>;

pub async fn is_exist(pool: &DbPool, id: i64) -> Option<bool> {
    let r = sqlx::query!("SELECT exists(select 1 FROM traders WHERE id = $1 LIMIT 1)", id)
        .fetch_one(pool).await;
    match r {
        Ok(r) => r.exists,
        Err(_) => None
    }
}
