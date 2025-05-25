mod agent;
mod cli;
mod config;
mod gemini;
mod persona;
mod task_part;

use crate::agent::Agent;
use crate::cli::Args;
use clap::Parser;

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

    // Initialize agent and run task
    let mut agent = Agent::new(api_key);
    agent.set_persona(persona)?;
    agent.run(&task).await?;

    Ok(())
}
