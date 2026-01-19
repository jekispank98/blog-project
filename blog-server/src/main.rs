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

    // Здесь будет код запуска сервера

    Ok(())
}

fn setup_environment() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}

