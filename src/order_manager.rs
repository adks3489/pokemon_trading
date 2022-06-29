use std::{collections::{BTreeMap, VecDeque}};
use futures::future;

use crate::{card, order_store};

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

#[derive(Debug, Clone, PartialEq)]
pub struct PendingOrder {
    pub id: i64,
    pub side: Action,
    // Cents, Shift decimal point by 2, 100 stand for 1.00 USD, 1000 stand for 10.00 USD, etc.
    pub price: i32,
    pub card_id: i32,
}
impl Eq for PendingOrder{}

type PriceBucket = VecDeque<PendingOrder>;

struct OrderBook {
    bids: BTreeMap<i32, PriceBucket>,
    asks: BTreeMap<i32, PriceBucket>
}
impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new()
        }
    }
    fn from_db(bids: Vec<order_store::PendingOrder>, asks: Vec<order_store::PendingOrder>) -> Self {
        let mut bids_tree = BTreeMap::new();
        bids.iter().for_each(|bid| {
            let price_bucket = bids_tree.entry(bid.price).or_insert(PriceBucket::new());
            price_bucket.push_back(PendingOrder {
                id: bid.id,
                side: Action::Buy,
                price: bid.price,
                card_id: bid.card_id,
            });
        });
        let mut asks_tree = BTreeMap::new();
        asks.iter().for_each(|ask| {
            let price_bucket = asks_tree.entry(ask.price).or_insert(PriceBucket::new());
            price_bucket.push_back(PendingOrder {
                id: ask.id,
                side: Action::Sell,
                price: ask.price,
                card_id: ask.card_id,
            });
        });
        OrderBook {
            bids: bids_tree,
            asks: asks_tree
        }
    }
    fn try_match(&mut self, order: &PendingOrder) -> Option<PendingOrder> {
        match order.side {
            Action::Buy => {
                let order_book = &mut self.asks;
                let best_price = *order_book.keys().nth(0)?;
                let mut matched_order = None;
                let price_bucket = order_book.get_mut(&best_price).expect("best price should exist");
                if order.price >= best_price {
                    matched_order = Some(price_bucket.pop_front().expect("price bucket should not be empty"));
                }
                if price_bucket.is_empty() {
                    order_book.remove(&best_price);
                }
                matched_order
            },
            Action::Sell => {
                let order_book = &mut self.bids;
                let best_price = *order_book.keys().last()?;
                let mut matched_order = None;
                let price_bucket = order_book.get_mut(&best_price).expect("best price should exist");
                if order.price <= best_price {
                    matched_order = Some(price_bucket.pop_front().expect("price bucket should not be empty"));
                }
                if price_bucket.is_empty() {
                    order_book.remove(&best_price);
                }
                matched_order
            }
        }
    }
    fn add_order(&mut self, order: PendingOrder) {
        match order.side {
            Action::Buy => {
                let order_book = &mut self.bids;
                let price_bucket = order_book.entry(order.price).or_insert(PriceBucket::new());
                price_bucket.push_back(order);
            },
            Action::Sell => {
                let order_book = &mut self.asks;
                let price_bucket = order_book.entry(order.price).or_insert(PriceBucket::new());
                price_bucket.push_back(order);
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FilledOrder {
    pub buy_order: i64,
    pub sell_order: i64,
    pub price: i32,
    pub card_id: i32,
    pub first_order_id: i64,
}
impl FilledOrder {
    fn new(pending_order: &PendingOrder, new_order_id: i64) -> Self {
        let buy_order;
        let sell_order;
        match pending_order.side {
            Action::Buy => {
                buy_order = pending_order.id;
                sell_order = new_order_id;
            },
            Action::Sell => {
                buy_order = new_order_id;
                sell_order = pending_order.id;
            }
        }
        FilledOrder {
            buy_order,
            sell_order,
            price: pending_order.price,
            card_id: pending_order.card_id,
            first_order_id: pending_order.id,
        }
    }
}

// TODO: may use threads per card to increase performance
pub struct OrderManager {
    order_books: Vec<OrderBook>,
    // TODO: let id decide by db, to support multiple manager
    last_id: i64,
}

impl OrderManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        OrderManager {
            order_books: (0..card::NUM_CARDS).map(|_| OrderBook::new()).collect(),
            last_id: 0,
        }
    }
    pub async fn from_db(db_pool: &order_store::DbPool) -> Self {
        let mut order_books = Vec::new();
        // TODO: join all futures from all cards
        for card_id in 0..card::NUM_CARDS {
            let bids = order_store::query_pending_orders(db_pool, card_id as i32, 0);
            let asks = order_store::query_pending_orders(db_pool, card_id as i32, 1);
            let (bids, asks) = future::join(bids, asks).await;
            let order_book = OrderBook::from_db(bids.unwrap(), asks.unwrap());
            order_books.push(order_book);
        }
        let last_id = order_store::query_last_id(db_pool).await.unwrap().id;
        OrderManager {
            order_books,
            last_id,
        }
    }

    pub fn take_id(&mut self) -> i64 {
        self.last_id += 1;
        self.last_id
    }
    pub fn add_order(&mut self, order: PendingOrder) -> Option<FilledOrder> {
        let matched_order = self.order_books[order.card_id as usize].try_match(&order);
        match matched_order {
            Some(o) => {
                Some(FilledOrder::new(&o, order.id))
            }
            None => {
                self.order_books[order.card_id as usize].add_order(order);
                None
            }
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_match_with_same_price() {
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Sell,
            price: 100,
            card_id: 0,
        };
        let filled_order = order_manager.add_order(order2);
        match filled_order {
            Some(o) => {         
                assert_eq!(FilledOrder{buy_order: 1, sell_order: 2, price: 100, card_id: 0, first_order_id: 1}, o);
            },
            None => assert!(false),
        }
    }

    #[test]
    fn test_match_with_first_come_order() {
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order2).is_none());
        let order3 = PendingOrder {
            id: 3,
            side: Action::Sell,
            price: 100,
            card_id: 0,
        };
        let filled_order = order_manager.add_order(order3);
        match filled_order {
            Some(o) => {         
                assert_eq!(FilledOrder{buy_order: 1, sell_order: 3, price: 100, card_id: 0, first_order_id: 1}, o);
            },
            None => assert!(false),
        }
    }

    #[test]
    fn test_different_card_not_matching() {
        // different card_id should not affect each other
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Sell,
            price: 100,
            card_id: 1,
        };
        assert!(order_manager.add_order(order2).is_none());
    }

    #[test]
    fn test_not_match_with_higher_sell_price() {
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Sell,
            price: 101,
            card_id: 0,
        };
        assert!(order_manager.add_order(order2).is_none());
    }

    #[test]
    fn test_not_match_with_lower_buy_price() {
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Sell,
            price: 101,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order2).is_none());
    }

    #[test]
    fn test_higher_buy_price_should_be_matched_first() {
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Buy,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Buy,
            price: 102,
            card_id: 0,
        };
        assert!(order_manager.add_order(order2).is_none());
        let order3 = PendingOrder {
            id: 3,
            side: Action::Sell,
            price: 99,
            card_id: 0,
        };        
        let filled_order = order_manager.add_order(order3);
        match filled_order {
            Some(o) => {         
                assert_eq!(FilledOrder{buy_order: 2, sell_order: 3, price: 102, card_id: 0, first_order_id: 2}, o);
            },
            None => assert!(false),
        }
    }

    #[test]
    fn test_lower_sell_price_should_be_matched_first() {
        let mut order_manager = OrderManager::new();
        let order1 = PendingOrder {
            id: 1,
            side: Action::Sell,
            price: 101,
            card_id: 0,
        };
        assert!(order_manager.add_order(order1).is_none());
        let order2 = PendingOrder {
            id: 2,
            side: Action::Sell,
            price: 100,
            card_id: 0,
        };
        assert!(order_manager.add_order(order2).is_none());
        let order3 = PendingOrder {
            id: 3,
            side: Action::Buy,
            price: 101,
            card_id: 0,
        };        
        let filled_order = order_manager.add_order(order3);
        match filled_order {
            Some(o) => {         
                assert_eq!(FilledOrder{buy_order: 3, sell_order: 2, price: 100, card_id: 0, first_order_id: 2}, o);
            },
            None => assert!(false),
        }
    }
}