use std::collections::{BTreeMap, VecDeque};

use crate::card;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Buy,
    Sell,
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
    pub id: u32,
    pub side: Action,
    // Cents, Shift decimal point by 2, 100 stand for 1.00 USD, 1000 stand for 10.00 USD, etc.
    pub price: u32,
    pub card_id: u32,
}
impl Eq for PendingOrder{}

type PriceBucket = VecDeque<PendingOrder>;

struct OrderBook {
    bids: BTreeMap<u32, PriceBucket>,
    asks: BTreeMap<u32, PriceBucket>
}
impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FilledOrder {
    pub buy_order: u32,
    pub sell_order: u32,
    pub price: u32,
    pub card_id: u32,
}
impl FilledOrder {
    fn new(pending_order: &PendingOrder, new_order_id: u32) -> Self {
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
        }
    }
}

// TODO: may use threads per card to increase performance
pub struct OrderManager {
    order_books: [OrderBook; card::NUM_CARDS],
    // TODO: use another id format to avoid run out of u32
    last_id: u32,
}

impl OrderManager {
    pub fn new() -> Self {
        // TODO: load from db
        OrderManager {
            order_books: [
                OrderBook::new(),
                OrderBook::new(),
                OrderBook::new(),
                OrderBook::new(),
            ],
            last_id: 0,
        }
    }
    pub fn take_id(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }
    pub fn add_order(&mut self, order: PendingOrder) -> Option<FilledOrder> {
        let matched_order = self.match_orderbook(&order);
        match matched_order {
            Some(o) => {
                Some(FilledOrder::new(&o, order.id))
            }
            None => {
                match order.side {
                    Action::Buy => {
                        let order_book = &mut self.order_books[order.card_id as usize].bids;
                        let price_bucket = order_book.entry(order.price).or_insert(PriceBucket::new());
                        price_bucket.push_back(order);
                    },
                    Action::Sell => {
                        let order_book = &mut self.order_books[order.card_id as usize].asks;
                        let price_bucket = order_book.entry(order.price).or_insert(PriceBucket::new());
                        price_bucket.push_back(order);
                    },
                }
                None
            }
        }
    }

    fn match_orderbook(&mut self, order: &PendingOrder) -> Option<PendingOrder> {
        match order.side {
            Action::Buy => {
                let order_book = &mut self.order_books[order.card_id as usize].asks;
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
                let order_book = &mut self.order_books[order.card_id as usize].bids;
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
                assert_eq!(FilledOrder{buy_order: 1, sell_order: 2, price: 100, card_id: 0}, o);
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
                assert_eq!(FilledOrder{buy_order: 1, sell_order: 3, price: 100, card_id: 0}, o);
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
                assert_eq!(FilledOrder{buy_order: 2, sell_order: 3, price: 102, card_id: 0}, o);
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
                assert_eq!(FilledOrder{buy_order: 3, sell_order: 2, price: 100, card_id: 0}, o);
            },
            None => assert!(false),
        }
    }
}