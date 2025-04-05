use clap::Parser;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The task to send to Gemini
    #[arg(short, long)]
    pub task: Option<String>,
}

impl Args {
    pub fn get_task(&self) -> io::Result<String> {
        Ok(match &self.task {
            Some(task) => task.clone(),
            None => {
                print!("Enter your task: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            }
        })
    }
} 