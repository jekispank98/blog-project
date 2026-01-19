mod server;
mod handlers;

pub mod blog {
    tonic::include_proto!("blog");
}

fn main() {
    println!("Starting blog-server...");
    server::run();
}
