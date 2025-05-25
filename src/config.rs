use directories::ProjectDirs;
use serde::Deserialize;
use std::{fs, path::PathBuf};

pub const ORG_NAME: &str = "light";
pub const APP_NAME: &str = "Fash CLI";

#[derive(Deserialize, Default, Debug)]
struct ConfigRaw {
    system_prompt: Option<SystemPrompt>,
}

#[derive(Debug)]
pub struct Config {
    pub system_prompt: Option<SystemPrompt>,
    proj_dirs: ProjectDirs,
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
                directories::ProjectDirs::from("com", ORG_NAME, APP_NAME)
                    .expect("Failed to create default config directory")
            });

        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");

        let config_raw = {
            if !config_path.exists() {
                ConfigRaw::default()
            } else {
                match fs::read_to_string(&config_path) {
                    Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                        eprintln!("Warning: Failed to parse config file: {}", e);
                        ConfigRaw::default()
                    }),
                    Err(e) => {
                        eprintln!("Warning: Failed to read config file: {}", e);
                        ConfigRaw::default()
                    }
                }
            }
        };
        Config {
            system_prompt: config_raw.system_prompt,
            proj_dirs,
        }
    }

    pub fn persona_dir(&self) -> PathBuf {
        self.proj_dirs.data_dir().join("personas")
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
