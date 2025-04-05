use clap::Parser;
use serde_json::json;
use std::io::{self, Write};
use serde::Deserialize;
use std::fs;

const ORG_NAME: &str = "light";
const APP_NAME: &str = "Fash CLI";

#[derive(Deserialize, Debug, Default)]
struct Config {
    system_prompt: Option<SystemPrompt>,
}

#[derive(Deserialize, Debug)]
struct SystemPrompt {
    file_path: Option<String>,
    inline_text: Option<String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The task to send to Gemini
    #[arg(short, long)]
    task: Option<String>,
}

fn load_config() -> Config {
    let proj_dirs = directories::ProjectDirs::from("com", ORG_NAME, APP_NAME)
        .unwrap_or_else(|| {
            eprintln!("Warning: Could not determine config directory, using defaults");
            return directories::ProjectDirs::from("com", ORG_NAME, APP_NAME)
                .expect("Failed to create default config directory");
        });

    let config_dir = proj_dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        return Config::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to parse config file: {}", e);
            Config::default()
        }),
        Err(e) => {
            eprintln!("Warning: Failed to read config file: {}", e);
            Config::default()
        }
    }
}

fn get_system_prompt(config: &Config) -> String {
    if let Some(system_prompt) = &config.system_prompt {
        // Try file path first
        if let Some(file_path) = &system_prompt.file_path {
            if let Ok(content) = fs::read_to_string(file_path) {
                return content;
            }
        }
        // Try inline text
        if let Some(text) = &system_prompt.inline_text {
            return text.clone();
        }
    }
    // Default system prompt
    "You are a helpful AI assistant.".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();
    
    // Load config
    let config = load_config();
    let system_prompt = get_system_prompt(&config);
    
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
                    "text": format!("Your name is fash.\n{}", system_prompt)
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
