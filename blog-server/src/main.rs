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
    setup_environment();
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "127.0.0.1:50051".parse()?;
    let service = ExchangeServiceImpl::new();
}

fn setup_environment() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}

