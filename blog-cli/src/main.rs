use clap::{Parser, Subcommand};
use blog_client::grpc_client::GrpcClient;
use blog_client::http_client::HttpClient;

#[derive(Parser)]
#[command(name = "blog-cli", version, about = "CLI client for the blog project")] 
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get a post by ID via HTTP
    HttpGet { id: String },
    /// Get a post by ID via gRPC
    GrpcGet { id: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::HttpGet { id } => {
            let client = HttpClient::new();
            let res = client.get_post(&id);
            println!("{}", res);
        }
        Commands::GrpcGet { id } => {
            let client = GrpcClient::new();
            let res = client.get_post(&id);
            println!("{}", res);
        }
    }
}
