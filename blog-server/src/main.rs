#[warn(missing_docs)]
use crate::infrastructure::database::create_pool;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use sqlx::migrate::Migrator;
use std::path::Path;
use std::sync::Arc;
use actix_web::middleware::Logger;
use crate::handlers::{configure, AuthService, BlogService};
use crate::infrastructure::config::Config;
use crate::infrastructure::jwt::Jwt;
use crate::presentation::grpc_service::BlogGrpcService;
use crate::blog::blog_service_server::BlogServiceServer;
use tonic::transport::Server;

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
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("DB: {:?}", env::var("DATABASE_URL"));
    let cfg = Config::from_env().expect("invalid config");
    let pool = create_pool().await.expect("Error creating database pool");

    let migrator = Migrator::new(Path::new("./blog-server/migrations")).await.expect("Failed to build migrator");
    migrator.run(&pool).await.expect("Failed to run migrations");

    let jwt_manager = Arc::new(Jwt::new());
    let auth_service = Arc::new(AuthService::new(pool.clone(), jwt_manager.clone()));
    let blog_service = Arc::new(BlogService::new(pool.clone()));

    let grpc_service = BlogGrpcService::new(
        blog_service.clone(),
        auth_service.clone(),
        jwt_manager.clone(),
    );

    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let grpc_server = Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(grpc_addr);

    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // Для разработки
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(blog_service.clone()))
            .app_data(web::Data::new(jwt_manager.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(configure)
    })
    .bind("127.0.0.1:8888")?
    .run();

    println!("Starting HTTP server on 127.0.0.1:8888");
    println!("Starting gRPC server on 0.0.0.0:50051");

    let http_handle = tokio::spawn(async move {
        http_server.await
    });

    let grpc_handle = tokio::spawn(async move {
        grpc_server.await
    });

    // Ждем их (они будут работать вечно)
    let _ = tokio::try_join!(http_handle, grpc_handle);
    /*tokio::select! {
        res = grpc_server => {
            if let Err(e) = res {
                eprintln!("gRPC server error: {}", e);
            }
        },
        res = http_server => {
            if let Err(e) = res {
                eprintln!("HTTP server error: {}", e);
            }
        }
    }*/

    Ok(())
}

