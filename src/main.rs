use std::sync::Mutex;
use actix_web::{web, get, post, delete, App, HttpResponse, HttpServer, Responder};
use log::{info, LevelFilter};
use envconfig::Envconfig;
use dotenv::dotenv;

use logger::Logger;
use config::Config;
use order_manager::OrderManager;
use serde::Deserialize;
mod card;
mod order_manager;
mod logger;
mod config;

#[get("/api/traders/{id}/orders")]
async fn get_orders() -> impl Responder {
    HttpResponse::Ok().body("[]")
}

#[derive(Deserialize)]
struct OrderRequest {
    side: String,
    price: u32,
    card_id: u32,
}
#[post("/api/traders/{id}/orders")]
async fn add_order(data: web::Data<Service>, req_body: web::Json<OrderRequest>) -> impl Responder {
    let side = order_manager::Action::from_str(&req_body.side);
    if side.is_none() {
        return HttpResponse::BadRequest().body("Invalid order side");
    }
    if req_body.price < 100 || req_body.price > 1000 {
        return HttpResponse::BadRequest().body("Price must be in the range of 100 to 1000 cents");
    }
    if card::is_valid(req_body.card_id) {
        return HttpResponse::BadRequest().body("Invalid card id");
    }
    
    let mut order_manager = data.order_manager.lock().unwrap();
    let id = order_manager.take_id();
    order_manager.add_order(order_manager::PendingOrder {
        id,
        side: side.unwrap(),
        price: req_body.price,
        card_id: req_body.card_id,
    });
    HttpResponse::Accepted().body("")
}

#[delete("/api/traders/{id}/orders/{order_id}")]
async fn delete_order() -> impl Responder {
    // TODO: delete order
    // TODO: can only delete unfilled order
    HttpResponse::NoContent().body("")
}

#[get("/api/cards/{id}/trades")]
async fn get_trades() -> impl Responder {
    // TODO:
    HttpResponse::Ok().body("[]")
}

fn init_logger() {
    static LOGGER: Logger = Logger;
    log::set_max_level(LevelFilter::Info);
    log::set_logger(&LOGGER).unwrap();
}

struct Service {
    order_manager: Mutex<OrderManager>,
}
impl Service {
    fn new() -> Self {
        Service {
            order_manager: Mutex::new(OrderManager::new()),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = Config::init_from_env().expect("Load config failed");
    init_logger();
    info!("Server is starting...");
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Service::new()))
            .service(get_orders)
            .service(add_order)
            .service(delete_order)
            .service(get_trades)
    })
    .bind((config.host, config.port))?
    .run()
    .await
}
