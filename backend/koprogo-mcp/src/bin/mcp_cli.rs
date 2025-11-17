// MCP CLI - Command-line interface for KoproGo MCP
use clap::{Parser, Subcommand};
use koprogo_mcp::*;

#[derive(Parser)]
#[command(name = "koprogo-mcp")]
#[command(about = "KoproGo Model Context Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a chat message to an AI model
    Chat {
        /// Model to use (e.g., llama3:8b, claude-3-opus)
        #[arg(short, long)]
        model: String,

        /// Message to send
        message: String,

        /// Context (e.g., copro:123)
        #[arg(short, long)]
        context: Option<String>,

        /// Edge node URL (default: http://localhost:3031)
        #[arg(short, long, default_value = "http://localhost:3031")]
        edge_url: String,
    },

    /// List available models
    Models {
        /// Edge node URL (default: http://localhost:3031)
        #[arg(short, long, default_value = "http://localhost:3031")]
        edge_url: String,
    },

    /// Check health status
    Health {
        /// Edge node URL (default: http://localhost:3031)
        #[arg(short, long, default_value = "http://localhost:3031")]
        edge_url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Chat {
            model,
            message,
            context,
            edge_url,
        } => {
            println!("🤖 Sending message to {}...", model);

            let edge_client = EdgeClient::new(vec![edge_url]);

            let request = McpRequest::new(
                model.clone(),
                vec![Message::user(message.clone())],
                context,
            )?;

            match edge_client.execute_on_edge(&request).await {
                Ok(response) => {
                    println!("\n📝 Response from {}:", model);
                    println!("{}", response.content);
                    println!("\n📊 Stats:");
                    println!("  - Tokens: {}", response.usage.total_tokens);
                    println!("  - Latency: {}ms", response.execution_info.latency_ms);
                    println!("  - CO₂: {:.4}g", response.calculate_co2_grams());
                }
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Models { edge_url } => {
            println!("📚 Fetching available models from {}...", edge_url);

            let client = reqwest::Client::new();
            let url = format!("{}/mcp/v1/models", edge_url);

            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let models: serde_json::Value = response.json().await?;
                        println!("\n✅ Available models:");
                        println!("{}", serde_json::to_string_pretty(&models)?);
                    } else {
                        eprintln!("❌ Failed to fetch models: {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Health { edge_url } => {
            println!("🏥 Checking health of {}...", edge_url);

            let edge_client = EdgeClient::new(vec![edge_url.clone()]);
            let healths = edge_client.check_health().await;

            for health in healths {
                println!("\n🖥️  Node: {}", health.node_url);
                println!("  Status: {}", if health.is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });
                println!("  Models: {:?}", health.models_loaded);
                println!("  Active requests: {}", health.active_requests);
                println!("  Memory: {}/{}MB", health.used_memory_mb, health.total_memory_mb);
            }
        }
    }

    Ok(())
}
