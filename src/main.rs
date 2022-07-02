use std::sync::{Arc};
use actix_web::{web, get, post, delete, App, HttpResponse, HttpServer, Responder, middleware};
use log::{info, error};
use envconfig::Envconfig;
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions};
use serde::Deserialize;

mod ports;
mod order_service;
mod card;
mod order_manager;
mod config;
mod trader_store;
mod order_store;
mod trade_store;
mod graphql;

use config::Config;
use ports::{TraderStore, OrderStore, TradeStore, OrderService};
use trader_store::PostgresTraderStoreImpl;
use order_store::PostgresOrderStoreImpl;
use trade_store::PostgresTradeStoreImpl;

#[get("/api/traders/{id}/orders")]
async fn get_orders(order_store: web::Data<PostgresOrderStoreImpl>, trader_store: web::Data<PostgresTraderStoreImpl>, path: web::Path<i64>) -> impl Responder {
    let trader_id = path.into_inner();
    let is_trader_exist = trader_store.is_exist(trader_id).await;
    if is_trader_exist.is_none() {
        return HttpResponse::InternalServerError().body("Failed to query trader");
    }
    if !is_trader_exist.unwrap() {
        return HttpResponse::BadRequest().body("Trader does not exist");
    }
    let r = order_store.query_orders(trader_id, Some(50)).await;
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

type OrderServiceImpl = order_service::OrderServiceImpl<
    trader_store::PostgresTraderStoreImpl,
    order_store::PostgresOrderStoreImpl,
    trade_store::PostgresTradeStoreImpl>;

#[post("/api/traders/{id}/orders")]
async fn add_order(order_service: web::Data<OrderServiceImpl>, path: web::Path<i64>, req_body: web::Json<OrderRequest>) -> impl Responder {
    let trader_id = path.into_inner();
    let side = ports::Action::from_str(&req_body.side);
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

    let r = order_service.add_order(trader_id, side, req_body.price, req_body.card_id).await;
    match r {
        Ok(_) => {
            HttpResponse::Ok().body("")
        },
        Err(e) => {
            error!("Failed to add order: {}", e);
            HttpResponse::InternalServerError().body("Failed to add order")
        },
    }
}

#[delete("/api/traders/{id}/orders/{order_id}")]
async fn delete_order() -> impl Responder {
    // TODO: delete order
    // TODO: can only delete unfilled order
    HttpResponse::NoContent().body("")
}

#[get("/api/cards/{id}/trades")]
async fn get_trades(trade_store: web::Data<PostgresTradeStoreImpl>, path: web::Path<i32>) -> impl Responder {
    let card_id = path.into_inner();    
    if !card::is_valid(card_id) {
        return HttpResponse::NotFound().body("Card not found");
    }
    let r = trade_store.query_trades(card_id, Some(50)).await;
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let config = Config::init_from_env().expect("Load config failed");
    info!("Server is starting...");
    let pool = Arc::new(PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url).await.unwrap());

    let trader_store = trader_store::PostgresTraderStoreImpl{pg_pool: pool.clone()};
    let order_store = order_store::PostgresOrderStoreImpl{pg_pool: pool.clone()};
    let trade_store = trade_store::PostgresTradeStoreImpl{pg_pool: pool.clone()};
    let order_service = order_service::OrderServiceImpl::new(
        trader_store.clone(), order_store.clone(), trade_store.clone()).await;

    info!("Listening on {}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(|cfg| graphql::endpoint::configure(cfg, order_store.clone(), trade_store.clone()))
            .app_data(web::Data::new(order_service.clone()))
            .app_data(web::Data::new(trader_store.clone()))
            .app_data(web::Data::new(order_store.clone()))
            .app_data(web::Data::new(trade_store.clone()))
            .service(health)
            .service(get_orders)
            .service(add_order)
            .service(delete_order)
            .service(get_trades)
            .wrap(actix_cors::Cors::permissive())
    })
    .bind((config.host, config.port))?
    .run()
    .await
}
