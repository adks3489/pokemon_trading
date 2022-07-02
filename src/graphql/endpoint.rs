use std::sync::Arc;
use actix_web::{web, get, HttpResponse, Responder, route};
use actix_web_lab::respond::Html;
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

use crate::graphql::schema::{Schema, create_schema};
use crate::order_store::PostgresOrderStoreImpl;
use crate::trade_store::PostgresTradeStoreImpl;

#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", None))
}

#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(st: web::Data<Schema>, data: web::Json<GraphQLRequest>) -> impl Responder {
    let user = data.execute(&st, &()).await;
    HttpResponse::Ok().json(user)
}

pub fn configure(cfg: &mut web::ServiceConfig, order_store: PostgresOrderStoreImpl, trade_store: PostgresTradeStoreImpl) {
  let schema = Arc::new(create_schema(order_store, trade_store));
  cfg.app_data(web::Data::from(schema.clone()))
    .service(graphql)
    .service(graphql_playground);
}