use actix_web::{get, post, delete, App, HttpResponse, HttpServer, Responder};
use log::{info, LevelFilter};
use crate::logger::Logger;
mod logger;

#[get("/api/traders/{id}/orders")]
async fn get_orders() -> impl Responder {
    HttpResponse::Ok().body("[]")
}

#[post("/api/traders/{id}/orders")]
async fn add_order(req_body: String) -> impl Responder {
    // TODO: validate order
    // generate order id
    // TODO: 202  
    // HttpResponse::Accepted().body("[]")
    HttpResponse::ServiceUnavailable()
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    info!("Server is starting...");
    HttpServer::new(|| {
        App::new()
            .service(get_orders)
            .service(add_order)
            .service(delete_order)
            .service(get_trades)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
