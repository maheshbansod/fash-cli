pub struct AgentTool {
    name: String,
    description: String,
    execution_command: String,
}

impl AgentTool {
    pub fn new(name: &str, description: &str, execution_command: &str) -> Self {
        Self { name: name.to_string(), description: description.to_string(), execution_command: execution_command.to_string() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn execution_command(&self) -> &str {
        &self.execution_command
    }
}