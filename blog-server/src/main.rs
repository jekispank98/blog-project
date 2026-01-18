mod server;
mod handlers;

fn main() {
    println!("Starting blog-server...");
    server::run();
}
