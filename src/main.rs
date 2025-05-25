mod agent;
mod agent_tool;
mod cli;
mod config;
mod gemini;
mod persona;
mod task_part;

use crate::agent::Agent;
use crate::cli::Args;
use clap::Parser;
use chrono::Utc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();

    // Parse args and get task
    let args = Args::parse();
    let task = args.get_task()?;
    let persona = args.persona;

    // Get API key from environment
    let api_key =
        std::env::var("GEMINI_API_KEY").expect("Please set GEMINI_API_KEY in your .env file");
    
    let random_number = rand::random::<u32>();
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let my_log_file = std::fs::File::create(format!("fash_{}_{}.log", random_number, timestamp))?;

    let tracing_subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(my_log_file)
        .finish();
    tracing::subscriber::set_global_default(tracing_subscriber)?;

    // Initialize agent and run task
    info!("Initializing agent with persona: {}", persona.clone().unwrap_or("None".to_string()));
    let mut agent = Agent::new(api_key);
    agent.set_persona(persona)?;
    agent.run(&task).await?;

    Ok(())
}
