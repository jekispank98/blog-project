use std::net::SocketAddr;
use std::path::Path;
use actix_web::{web, App, HttpServer};
use sqlx::migrate;
use sqlx::migrate::Migrator;
use crate::infrastructure::database::create_pool;

mod server;
mod handlers;
mod domain;
pub mod data;
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
    let pool = create_pool().await.expect("Error creating database pool");
    let migrator = Migrator::new(Path::new("./migrations")).await.expect("Failed to build");
    migrator.run(&pool).await.expect("Failed to run migrations");
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

