use std::{fs, path::Path};

use std::error::Error;

use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct Persona {
    name: String,
    description: String,
    instructions: String,
    allow_personas_as_tools: Option<bool>,
}

impl Persona {
    /// Loads a persona from the file at the given path.
    /// The file is toml format with the following fields:
    /// - name: The name of the persona
    /// - description: A short description of the persona
    /// - instructions: The instructions for the persona
    ///
    pub fn load(persona_file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let persona_file_path = persona_file_path.with_extension("toml");
        let persona = fs::read_to_string(persona_file_path)?;
        let persona: Persona = toml::from_str(&persona)?;
        Ok(persona)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    pub fn allow_personas_as_tools(&self) -> bool {
        self.allow_personas_as_tools.unwrap_or(false)
    }
}