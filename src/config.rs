use serde::Deserialize;
use std::fs;

pub const ORG_NAME: &str = "light";
pub const APP_NAME: &str = "Fash CLI";

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub system_prompt: Option<SystemPrompt>,
}

#[derive(Deserialize, Debug)]
pub struct SystemPrompt {
    pub file_path: Option<String>,
    pub inline_text: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let proj_dirs =
            directories::ProjectDirs::from("com", ORG_NAME, APP_NAME).unwrap_or_else(|| {
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

    pub fn get_system_prompt(&self) -> String {
        if let Some(system_prompt) = &self.system_prompt {
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
}
