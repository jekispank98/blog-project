use std::net::SocketAddr;
use actix_web::{web, App, HttpServer};

mod server;
mod handlers;
mod domain;
mod data;
mod application;
mod infrastructure;
mod presentation;

pub mod blog {
    tonic::include_proto!("blog");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let _addr: SocketAddr = "127.0.0.1:50051".parse().expect("Invalid address");

    HttpServer::new(|| {
        App::new()
        // .route("/", web::get().to(...))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

fn setup_environment() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}

