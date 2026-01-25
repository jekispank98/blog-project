#[warn(missing_docs)]

use std::path::Path;
use clap::{Parser, Subcommand};
use tokio::fs;
use blog_client::{BlogClient, Transport};

const TOKEN_FILE: &str = ".blog_token";
#[derive(Parser)]
#[command(name = "blog-cli", version, about = "CLI client for the blog project")] 
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, global = true)]
    grpc: bool,
    #[arg(long)]
    server: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Register {
        #[arg(long)] username: String,
        #[arg(long)] email: String,
        #[arg(long)] password: String,
    },
    Login {
        #[arg(long)] username: String,
        #[arg(long)] password: String,
    },
    Create {
        #[arg(long)] title: String,
        #[arg(long)] content: String,
    },
    Get { id: String },
    Update {
        id: String,
        #[arg(long)] title: String,
        #[arg(long)] content: String,
    },
    Delete { id: String },
    List {
        #[arg(long, default_value_t = 10)] limit: i64,
        #[arg(long, default_value_t = 0)] offset: i64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let addr = cli.server.unwrap_or_else(|| {
        if cli.grpc { "http://127.0.0.1:50051".to_string() }
        else { "http://127.0.0.1:8888".to_string() }
    });
    println!("addr: {}", addr);

    let transport = if cli.grpc { Transport::Grpc(addr) }
    else { Transport::Http(addr) };


    let mut client = BlogClient::new(transport).await?;

    if Path::new(TOKEN_FILE).exists() {
        let token = fs::read_to_string(TOKEN_FILE).await?;
        client.set_token(token.trim().to_string());
    }

    match cli.command {
        Commands::Register { username, email, password } => {
            let res = client.register(username, email, password).await?;
            fs::write(TOKEN_FILE, &res.token).await?;
            println!("Registered! Token saved. User: {}", res.user.username);
        }
        Commands::Login { username, password } => {
            let res = client.login(username, password).await?;
            fs::write(TOKEN_FILE, &res.token).await?;
            println!("Logged in! Token saved.");
        }
        Commands::Create { title, content } => {
            let post = client.create_post(title, content).await?;
            println!("Created post: {:?}", post);
        }
        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            println!("Post: {:?}", post);
        }
        Commands::Update { id, title, content } => {
            let post = client.update_post(id, title, content).await?;
            println!("Updated post: {:?}", post);
        }
        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("Post deleted.");
        }
        Commands::List { limit, offset } => {
            let list = client.list_posts(limit, offset).await?;
            println!("Total: {}. Posts: {:?}", list.total, list.posts);
        }
    }

    Ok(())
}
