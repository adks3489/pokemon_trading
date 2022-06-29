use std::sync::{Mutex, Arc};
use actix_web::{web, get, post, delete, App, HttpResponse, HttpServer, Responder};
use log::{info, LevelFilter, error};
use envconfig::Envconfig;
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions};
use chrono::Utc;

use logger::Logger;
use config::Config;
use order_manager::OrderManager;
use serde::Deserialize;
mod card;
mod order_manager;
mod logger;
mod config;
mod traders_store;
mod order_store;
mod trades_store;

#[get("/api/traders/{id}/orders")]
async fn get_orders(db_pool: web::Data<order_store::DbPool>, path: web::Path<i64>) -> impl Responder {
    let trader_id = path.into_inner();    
    let is_trader_exist = traders_store::is_exist(&db_pool, trader_id).await;
    if is_trader_exist.is_none() {
        return HttpResponse::InternalServerError().body("Failed to query trader");
    }
    if !is_trader_exist.unwrap() {
        return HttpResponse::BadRequest().body("Trader does not exist");
    }
    let r = order_store::query_orders(&db_pool, trader_id, Some(50)).await;
    match r {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => {
            error!("Failed to query orders: {}", e);
            HttpResponse::InternalServerError().body("Failed to query orders")
        },
    }
}

#[derive(Deserialize)]
struct OrderRequest {
    side: String,
    price: i32,
    card_id: i32,
}
#[post("/api/traders/{id}/orders")]
async fn add_order(app: web::Data<Service>, db_pool: web::Data<order_store::DbPool>, path: web::Path<i64>, req_body: web::Json<OrderRequest>) -> impl Responder {
    // TODO: check trader id exist
    let trader_id = path.into_inner();
    let side = order_manager::Action::from_str(&req_body.side);
    if side.is_none() {
        return HttpResponse::BadRequest().body("Invalid order side");
    }
    if req_body.price < 100 || req_body.price > 1000 {
        return HttpResponse::BadRequest().body("Price must be in the range of 100 to 1000 cents");
    }
    if !card::is_valid(req_body.card_id) {
        return HttpResponse::BadRequest().body("Invalid card id");
    }
    let side = side.unwrap();
    info!("Received order request: {:?} card={} price={}", &side, &req_body.card_id, req_body.price);

    let is_trader_exist = traders_store::is_exist(&db_pool, trader_id).await;
    if is_trader_exist.is_none() {
        return HttpResponse::InternalServerError().body("Failed to query trader");
    }
    if !is_trader_exist.unwrap() {
        return HttpResponse::BadRequest().body("Trader does not exist");
    }

    let mut order_manager = app.order_manager.lock().unwrap();
    let order_id = order_manager.take_id();
    let filled_order = order_manager.add_order(order_manager::PendingOrder {
        id: order_id,
        side: side.clone(),
        price: req_body.price,
        card_id: req_body.card_id,
    });
    match filled_order {
        Some(order) => {
            let transaction = db_pool.begin().await;
            let transaction = match transaction {
                Ok(transaction) => transaction,
                Err(e) => {
                    error!("Failed to begin transaction: {}", e);
                    return HttpResponse::InternalServerError().body("Failed to finish order");
                }
            };
            let r = order_store::insert_order(&db_pool, order_store::Order{
                id: order_id,
                card_id: req_body.card_id,
                price: req_body.price,
                side: side as i16,
                status: order_store::Status::Filled as i16,
                trader_id,
                created_at: Utc::now(),
            }).await;
            if let Err(e) = r {
                error!("Failed to insert order: {}", e);
                return HttpResponse::InternalServerError().body("Failed to insert order");
            }
            let r = order_store::update_order_status(&db_pool, order.first_order_id, order_store::Status::Filled).await;            
            if let Err(e) = r {
                error!("Failed to update order status: {}", e);
                return HttpResponse::InternalServerError().body("Failed to update order status");
            }
            let r = trades_store::insert_trade(&db_pool, order.card_id, order.price, order.buy_order, order.sell_order).await;
            if let Err(e) = r {
                error!("Failed to insert trade: {}", e);
                return HttpResponse::InternalServerError().body("Failed to insert trade");
            }
            let r = transaction.commit().await;
            if let Err(e) = r {
                error!("Failed to commit transaction: {}", e);
                return HttpResponse::InternalServerError().body("Failed to finish order");
            }
        },
        None => {
            let r = order_store::insert_order(&db_pool, order_store::Order{
                id: order_id,
                card_id: req_body.card_id,
                price: req_body.price,
                side: side as i16,
                status: order_store::Status::Pending as i16,
                trader_id,
                created_at: Utc::now(),
            }).await;
            if let Err(e) = r {
                error!("Failed to insert order: {}", e);
                return HttpResponse::InternalServerError().body("Failed to insert order");
            }
        },
    }
    HttpResponse::Ok().body("")
}

#[delete("/api/traders/{id}/orders/{order_id}")]
async fn delete_order() -> impl Responder {
    // TODO: delete order
    // TODO: can only delete unfilled order
    HttpResponse::NoContent().body("")
}

#[get("/api/cards/{id}/trades")]
async fn get_trades(db_pool: web::Data<trades_store::DbPool>, path: web::Path<i32>) -> impl Responder {
    let card_id = path.into_inner();    
    if !card::is_valid(card_id) {
        return HttpResponse::NotFound().body("Card not found");
    }
    let r = trades_store::query_trades(&db_pool, card_id, Some(50)).await;
    match r {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => {
            error!("Failed to query trades: {}", e);
            HttpResponse::InternalServerError().body("Failed to query trades")
        },
    }
}

#[get("/api/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("alive")
}

fn init_logger() {
    static LOGGER: Logger = Logger;
    log::set_max_level(LevelFilter::Info);
    log::set_logger(&LOGGER).unwrap();
}

#[derive(Clone)]
struct Service {
    order_manager: Arc<Mutex<OrderManager>>,
}
impl Service {
    async fn new(db_pool: &order_store::DbPool) -> Self {
        Service {
            order_manager: Arc::new(Mutex::new(OrderManager::from_db(db_pool).await)),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = Config::init_from_env().expect("Load config failed");
    init_logger();
    info!("Server is starting...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url).await.unwrap();
    let service = Service::new(&pool).await;
    info!("Listening on {}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(health)
            .service(get_orders)
            .service(add_order)
            .service(delete_order)
            .service(get_trades)
    })
    .bind((config.host, config.port))?
    .run()
    .await
}
