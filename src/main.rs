use clap::Parser;
use serde_json::json;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The task to send to Gemini
    #[arg(short, long)]
    task: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();
    
    let args = Args::parse();
    
    // Get the task either from command line or interactively
    let task = match args.task {
        Some(task) => task,
        None => {
            print!("Enter your task: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    // Get API key from environment
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("Please set GEMINI_API_KEY in your .env file");

    // Prepare the request to Gemini API
    let client = reqwest::Client::new();
    let response = client
        .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent")
        .query(&[("key", api_key)])
        .json(&json!({
            "system_instruction": {
                "parts": [{
                    "text": "You are fash"
                }]
            },
            "contents": [{
                "parts": [{
                    "text": task
                }]
            }]
        }))
        .send()
        .await?;

    // Parse and display the response
    let result = response.json::<serde_json::Value>().await?;
    
    if let Some(text) = result
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
    {
        println!("\nGemini's response:\n{}", text);
    } else {
        println!("Error: Unexpected response format from Gemini API");
        println!("Raw response: {}", result);
    }

    Ok(())
}
