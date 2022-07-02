use async_trait::async_trait;
use anyhow::{Result};
use serde::{Serialize};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TraderStore {
    async fn is_exist(&self, id: i64) -> Option<bool>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(i16)]
pub enum Action {
    Buy = 0,
    Sell = 1,
}
impl Action {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "buy" => Some(Action::Buy),
            "sell" => Some(Action::Sell),
            _ => None,
        }
    }
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

pub struct NewOrder {
  pub card_id: i32,
  pub price: i32,
  pub action: Action,
  pub status: i16,
  pub trader_id: i64,
  pub created_at: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i16)]
pub enum Status {
    Pending = 0,
    Filled = 1,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct PendingOrder {
    pub id: i64,
    pub side: i16,
    pub price: i32,
    pub card_id: i32,
    // TODO: only id & price is must-have
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderStore {
  async fn query_orders(&self, trader_id: i64, limit: Option<i64>) -> Result<Vec<Order>>;
  async fn insert_order(&self, order: NewOrder) -> Result<i64>;
  async fn update_order_status(&self, order_id: i64, status: Status) -> Result<()>;
  async fn query_pending_orders(&self, card_id: i32, side: i16) -> Result<Vec<PendingOrder>>;
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

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TradeStore {
  async fn insert_trade(&self, card_id: i32, price: i32, buyorder_id: i64, sellorder_id: i64) -> Result<()>;
  async fn query_trades(&self, card_id: i32, limit: Option<i64>) -> Result<Vec<Trade>>;
}


#[async_trait]
pub trait OrderService {
    async fn add_order(&self, trader_id: i64, side: Action, price: i32, card_id: i32) -> Result<()>;
}