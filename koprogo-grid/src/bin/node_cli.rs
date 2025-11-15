use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::time;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "koprogo-grid-node")]
#[command(about = "KoproGo Grid Computing Node CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register this node with the grid
    Register {
        /// Grid server URL
        #[arg(short, long, default_value = "http://localhost:8081")]
        server: String,

        /// Node name
        #[arg(short, long)]
        name: String,

        /// Number of CPU cores to dedicate
        #[arg(short, long, default_value = "4")]
        cores: u32,

        /// Does this node have solar panels?
        #[arg(long)]
        solar: bool,

        /// Node location
        #[arg(short, long, default_value = "Unknown")]
        location: String,
    },

    /// Run the node worker (heartbeat + task execution)
    Run {
        /// Grid server URL
        #[arg(short, long, default_value = "http://localhost:8081")]
        server: String,

        /// Node ID (from registration)
        #[arg(short, long)]
        node_id: String,

        /// Simulated solar watts (for testing)
        #[arg(long, default_value = "0")]
        solar_watts: f64,

        /// Heartbeat interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cli = Cli::parse();

    match cli.command {
        Commands::Register {
            server,
            name,
            cores,
            solar,
            location,
        } => {
            register_node(&server, &name, cores, solar, &location).await?;
        }
        Commands::Run {
            server,
            node_id,
            solar_watts,
            interval,
        } => {
            run_worker(&server, &node_id, solar_watts, interval).await?;
        }
    }

    Ok(())
}

async fn register_node(
    server: &str,
    name: &str,
    cores: u32,
    solar: bool,
    location: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let payload = json!({
        "name": name,
        "cpu_cores": cores,
        "has_solar": solar,
        "location": location
    });

    log::info!("Registering node with grid at {}", server);

    let response = client
        .post(format!("{}/grid/register", server))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        let node_id = result["id"].as_str().unwrap_or("unknown");

        log::info!("✅ Node registered successfully!");
        println!("\n🎉 Node registered successfully!");
        println!("📝 Node ID: {}", node_id);
        println!("\n💡 To run the worker, use:");
        println!(
            "   koprogo-grid-node run --server {} --node-id {}",
            server, node_id
        );
    } else {
        let error = response.text().await?;
        log::error!("❌ Registration failed: {}", error);
        println!("❌ Registration failed: {}", error);
    }

    Ok(())
}

async fn run_worker(
    server: &str,
    node_id: &str,
    solar_watts: f64,
    interval_secs: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let node_uuid = Uuid::parse_str(node_id)?;

    log::info!("Starting worker for node {}", node_id);
    println!("🚀 KoproGo Grid Node Worker starting...");
    println!("📡 Server: {}", server);
    println!("🆔 Node ID: {}", node_id);
    println!("⏱️  Heartbeat interval: {}s", interval_secs);
    println!("\n⚡ Worker running... (Ctrl+C to stop)\n");

    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );

    let mut interval = time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        // Get CPU usage
        sys.refresh_cpu();
        let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

        // Send heartbeat
        match send_heartbeat(&client, server, node_uuid, cpu_usage, solar_watts).await {
            Ok(eco_score) => {
                log::info!(
                    "💓 Heartbeat sent - CPU: {:.1}% | Solar: {:.0}W | Eco Score: {:.2}",
                    cpu_usage,
                    solar_watts,
                    eco_score
                );
            }
            Err(e) => {
                log::error!("❌ Heartbeat failed: {}", e);
            }
        }

        // Check for available tasks
        match get_task(&client, server, node_uuid).await {
            Ok(Some(task)) => {
                log::info!("📦 Received task: {} (type: {})", task.id, task.task_type);
                println!("\n📦 New task received!");
                println!("   ID: {}", task.id);
                println!("   Type: {}", task.task_type);
                println!("   Reward: €{:.4}", task.estimated_reward);

                // Simulate task execution
                execute_task(&client, server, &task, solar_watts).await?;
            }
            Ok(None) => {
                // No tasks available
            }
            Err(e) => {
                log::error!("❌ Task fetch failed: {}", e);
            }
        }
    }
}

async fn send_heartbeat(
    client: &Client,
    server: &str,
    node_id: Uuid,
    cpu_usage: f64,
    solar_watts: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    let payload = json!({
        "node_id": node_id,
        "cpu_usage": cpu_usage,
        "solar_watts": solar_watts
    });

    let response = client
        .post(format!("{}/grid/heartbeat", server))
        .json(&payload)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    let eco_score = result["eco_score"].as_f64().unwrap_or(0.0);

    Ok(eco_score)
}

#[derive(serde::Deserialize)]
struct TaskResponse {
    id: String,
    task_type: String,
    estimated_reward: f64,
}

async fn get_task(
    client: &Client,
    server: &str,
    node_id: Uuid,
) -> Result<Option<TaskResponse>, Box<dyn std::error::Error>> {
    let response = client
        .get(format!("{}/grid/task?node_id={}", server, node_id))
        .send()
        .await?;

    if response.status() == 204 {
        return Ok(None);
    }

    let task: TaskResponse = response.json().await?;
    Ok(Some(task))
}

async fn execute_task(
    client: &Client,
    server: &str,
    task: &TaskResponse,
    solar_watts: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("🔨 Executing task {}...", task.id);
    println!("   🔨 Executing task...");

    // Simulate task execution (in real implementation, would download data, process, upload result)
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Simulate energy usage (5-15 Wh)
    let energy_used = 10.0 + (rand::random::<f64>() * 5.0);

    // Calculate solar contribution (up to 100% of solar_watts for this duration)
    let solar_contribution = (solar_watts * 0.0014).min(energy_used); // ~5s worth

    // Generate a mock result hash
    let result_hash = format!("{:x}", md5::compute(task.id.as_bytes()));

    let payload = json!({
        "task_id": task.id,
        "result_hash": result_hash,
        "energy_used_wh": energy_used,
        "solar_contribution_wh": solar_contribution
    });

    let response = client
        .post(format!("{}/grid/report", server))
        .json(&payload)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    let carbon_credits = result["carbon_credits"].as_f64().unwrap_or(0.0);
    let node_share = result["node_share_eur"].as_f64().unwrap_or(0.0);
    let coop_share = result["cooperative_share_eur"].as_f64().unwrap_or(0.0);

    println!("   ✅ Task completed!");
    println!("   🌱 Carbon saved: {:.6} kg CO₂", carbon_credits);
    println!("   💰 Your share: €{:.6}", node_share);
    println!("   🤝 Cooperative fund: €{:.6}\n", coop_share);

    Ok(())
}
