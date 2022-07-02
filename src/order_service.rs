use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use anyhow::{anyhow, Result, Context};
use chrono::Utc;

use crate::ports::{OrderService, TraderStore, OrderStore, TradeStore, Status, NewOrder, Action};
use crate::order_manager::{self, OrderManager};

#[derive(Clone)]
pub struct OrderServiceImpl<A: TraderStore, B: OrderStore, C: TradeStore> {
  pub trader_store: A,
  pub order_store: B,
  pub trade_store: C,
  order_manager: Arc<Mutex<OrderManager>>,
}
impl <A, B, C> OrderServiceImpl<A, B, C>  
  where A: TraderStore + Sync + Send,
        B: OrderStore + Sync + Send,
        C: TradeStore + Sync + Send {
  pub async fn new(trader_store: A, order_store: B, trade_store: C) -> Self {
    let order_manager = Arc::new(Mutex::new(OrderManager::from_db(&order_store).await));
    Self {
      trader_store,
      order_store,
      trade_store,
      order_manager,
    }
  }
}

#[async_trait]
impl <A, B, C> OrderService for OrderServiceImpl<A, B, C> 
  where A: TraderStore + Sync + Send,
        B: OrderStore + Sync + Send,
        C: TradeStore + Sync + Send {
  async fn add_order(&self, trader_id: i64, side: Action, price: i32, card_id: i32) -> Result<()> {
    let is_trader_exist = self.trader_store.is_exist(trader_id).await;
    if is_trader_exist.is_none() || is_trader_exist.unwrap() == false {
      return Err(anyhow!("Trader not exist"));
    }

    let order_id = self.order_store.insert_order(NewOrder{
        card_id,
        price,
        action: side.clone(),
        status: Status::Pending as i16,
        trader_id,
        created_at: Utc::now(),
    }).await.with_context(|| "Insert order failed")?;

    let filled_order = self.order_manager.lock().unwrap().add_order(order_manager::PendingOrder {
        id: order_id,
        side,
        price,
        card_id,
    });

    if let Some(order) = filled_order {
        self.order_store.update_order_status( order_id, Status::Filled).await.with_context(|| format!("Failed to update order status: {}", order_id))?;
        self.order_store.update_order_status( order.first_order_id, Status::Filled).await.with_context(|| format!("Failed to update order status: {}", order.first_order_id))?;
        self.trade_store.insert_trade(order.card_id, order.price, order.buy_order, order.sell_order).await.with_context(|| format!("Failed to insert trade: {}", order_id))?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{ports::{MockTraderStore, MockOrderStore, MockTradeStore}};
  use super::*;

  #[actix_web::main]
  #[test]
  async fn test_add_order() {
    let mut trader_store = MockTraderStore::new();
    let mut order_store = MockOrderStore::new();
    let mut trade_store = MockTradeStore::new();
    trader_store.expect_is_exist().returning(|_| Some(true)).times(2);
    order_store.expect_query_pending_orders().returning(|_, _| Ok(vec![])).times(1..);
    order_store.expect_insert_order().returning(|_| Ok(1)).times(2);
    order_store.expect_update_order_status().returning(|_, _| Ok(())).times(2);
    trade_store.expect_insert_trade().returning(|_, _, _, _| Ok(())).times(1);

    let order_service = OrderServiceImpl::new(trader_store, order_store, trade_store).await;
    assert!(order_service.add_order(1, Action::Buy, 100, 1).await.is_ok());
    assert!(order_service.add_order(2, Action::Sell, 100, 1).await.is_ok());
  }
}