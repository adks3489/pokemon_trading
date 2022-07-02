use juniper::{FieldResult, EmptySubscription, RootNode, GraphQLObject};

use crate::ports::{self, TradeStore, OrderStore};
use crate::order_store::PostgresOrderStoreImpl;
use crate::trade_store::PostgresTradeStoreImpl;

#[derive(GraphQLObject)]
#[graphql(description = "Order")]
struct Order {
    pub id: String,
    pub card_id: i32,
    pub price: i32,
    pub side: i32,
    pub status: i32,
    pub trader_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>
}
impl From<ports::Order> for Order{
    fn from(order: ports::Order) -> Self {
        Order {
            id: order.id.to_string(),
            card_id: order.card_id,
            price: order.price,
            side: order.side.into(),
            status: order.status.into(),
            trader_id: order.trader_id.to_string(),
            created_at: order.created_at
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "Trade record")]
struct Trade {
    pub id: String,
    pub card_id: i32,
    pub price: i32,
    pub buyorder_id: String,
    pub sellorder_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>
}
impl From<ports::Trade> for Trade{
    fn from(trade: ports::Trade) -> Self {
        Trade {
            id: trade.id.to_string(),
            card_id: trade.card_id,
            price: trade.price,
            buyorder_id: trade.buyorder_id.to_string(),
            sellorder_id: trade.sellorder_id.to_string(),
            created_at: trade.created_at
        }
    }
}

pub struct QueryRoot{
    order_store: PostgresOrderStoreImpl,
    trade_store: PostgresTradeStoreImpl,
}

#[juniper::graphql_object]
impl QueryRoot {
    async fn trades(&self, card_id: i32) -> FieldResult<Vec<Trade>> {
        Ok(self.trade_store.query_trades(card_id, None).await?.into_iter().map(|trade| trade.into()).collect())
    }
    async fn orders(&self, trader_id: String) -> FieldResult<Vec<Order>> {
        Ok(self.order_store.query_orders(trader_id.parse::<i64>()?, None).await?.into_iter().map(|order| order.into()).collect())
    }
}

pub struct MutationRoot;

#[juniper::graphql_object]
impl MutationRoot {
    fn add_order() -> FieldResult<bool> {
        todo!();
        Ok(true)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(order_store: PostgresOrderStoreImpl, trade_store: PostgresTradeStoreImpl) -> Schema {
    Schema::new(QueryRoot {order_store, trade_store}, MutationRoot {}, EmptySubscription::new())
}