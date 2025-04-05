mod config;
mod cli;
mod gemini;

use clap::Parser;
use crate::cli::Args;
use crate::config::Config;
use crate::gemini::GeminiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();
    
    // Load config
    let config = Config::load();
    let system_prompt = config.get_system_prompt();
    
    // Parse args and get task
    let args = Args::parse();
    let task = args.get_task()?;

    // Get API key from environment
    // TODO: get this from config
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("Please set GEMINI_API_KEY in your .env file");

    // Initialize Gemini client and generate response
    let client = GeminiClient::new(api_key);
    let response = client.generate_content(&task, &system_prompt).await?;
    
    println!("\nGemini's response:\n{}", response);

    Ok(())
}
