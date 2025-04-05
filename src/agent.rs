use std::error::Error;
use crate::config::Config;
use crate::gemini::GeminiClient;

pub struct Agent {
    config: Config,
    client: GeminiClient,
}

impl Agent {
    pub fn new(api_key: String) -> Self {
        let config = Config::load();
        let client = GeminiClient::new(api_key);
        
        Self {
            config,
            client,
        }
    }
    
    pub async fn run(&self, task: &str) -> Result<String, Box<dyn Error>> {
        let system_prompt = self.config.get_system_prompt();
        let system_prompt = format!("You are fash.\n{}", system_prompt);
        self.client.generate_content(task, &system_prompt).await
    }
} 