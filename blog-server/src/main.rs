use crate::infrastructure::database::create_pool;
use actix_web::{web, App, HttpServer};
use sqlx::migrate::Migrator;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use actix_web::middleware::Logger;
use tracing::callsite::register;
use crate::handlers::{configure, AuthService, BlogService};
use crate::infrastructure::config::Config;
use crate::infrastructure::jwt::Jwt;

mod server;
pub mod handlers;
pub mod domain;
pub mod data;
mod application;
mod infrastructure;
pub mod presentation;

pub mod blog {
    tonic::include_proto!("blog");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let _addr: SocketAddr = "127.0.0.1:50051".parse().expect("Invalid address");
    let cfg = Config::from_env().expect("invalid config");
    let pool = create_pool().await.expect("Error creating database pool");
    let migrator = Migrator::new(Path::new("./migrations")).await.expect("Failed to build");
    migrator.run(&pool).await.expect("Failed to run migrations");
    let jwt_manager = Arc::new(Jwt::new());
    let auth_service = Arc::new(AuthService::new(pool.clone(), jwt_manager.clone()));
    let blog_service = Arc::new(BlogService::new(pool.clone()));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(blog_service.clone()))
            .app_data(web::Data::new(jwt_manager.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(configure)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

