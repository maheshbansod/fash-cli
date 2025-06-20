use std::error::Error;

use tracing::{info, instrument};

use crate::agent_tool::AgentTool;
use crate::config::Config;
use crate::gemini::GeminiClient;
use crate::persona::Persona;
use crate::task_part::TaskPart;

type Message = (&'static str, String);

pub struct Agent {
    config: Config,
    client: GeminiClient,
    persona: Option<Persona>,
    messages: Vec<Message>,
    tools: Vec<AgentTool>,
}

impl Agent {
    pub fn new(api_key: String) -> Self {
        let config = Config::load();
        let client = GeminiClient::new(api_key);

        Self {
            config,
            client,
            persona: None,
            messages: vec![],
            tools: vec![],
        }
    }

    pub fn set_persona(&mut self, persona: Option<String>) -> Result<(), Box<dyn Error>> {
        if let Some(persona) = persona {
            let persona_dir = self.config.persona_dir();
            let persona_file_path = persona_dir.join(persona);
            let persona = Persona::load(&persona_file_path)?;
            if persona.allow_personas_as_tools() {
                let persona_dir = self.config.persona_dir();
                // get all personas
                let persona_files = std::fs::read_dir(persona_dir)?;
                for persona_file in persona_files {
                    if persona_file.is_err() {
                        continue;
                    }
                    let persona_file = persona_file.unwrap();
                    let persona_file_path = persona_file.path();
                    let persona_file_name =
                        persona_file_path.file_stem().unwrap().to_str().unwrap();
                    if persona_file_name.starts_with(".") {
                        continue;
                    }
                    // i should cache this somehow i think
                    let persona = Persona::load(&persona_file_path)?;
                    self.tools.push(AgentTool::new(
                        persona.name(),
                        persona.description(),
                        // i should make this command dynamic -> agent-base can't always be the binary name
                        &format!(
                            "agent-base --persona {} --task \"<task>\"",
                            persona_file_name
                        ),
                    ));
                }
            }
            self.persona = Some(persona);
        }
        Ok(())
    }

    #[instrument]
    pub async fn run(&mut self, task: &str) -> Result<(), Box<dyn Error>> {
        let system_prompt = self.config.get_system_prompt();
        let response_format = self.response_format();
        self.messages = vec![("user", format!("The task is: {}", task))];
        let system_prompt = format!(
            "You are an instance of fash. You live at https://github.com/maheshbansod/fash-cli .
You are an autonomous agent that will be run in a terminal with very limited user interaction.

{system_prompt}

{}

{response_format}

{}",
            if let Some(persona) = &self.persona {
                format!(
                    "The persona you need to adopt is:
{} - {}
{}",
                    persona.name(),
                    persona.description(),
                    persona.instructions()
                )
            } else {
                String::new()
            },
            if self.tools.is_empty() {
                String::new()
            } else {
                format!(
                    "You can use the following tools:\n{}",
                    self.tools
                        .iter()
                        .map(|tool| format!(
                            "{}\n{}\nCommand: {}",
                            tool.name(),
                            tool.description(),
                            tool.execution_command()
                        ))
                        .collect::<Vec<String>>()
                        .join("\n\n")
                )
            }
        );
        let mut should_exit = false;
        while !should_exit {
            let response = self
                .client
                .generate_content(&self.messages, &system_prompt)
                .await?;
            // remove the first line of response if it starts with ``` and also remove the last ``` in the response
            let response = if let Some(stripped) = response.strip_prefix("```json") {
                stripped.to_string()
            } else {
                response
            };
            let response = if let Some(stripped) = response.strip_prefix("```") {
                stripped.to_string()
            } else {
                response
            };
            let response = if let Some(stripped) = response.strip_suffix("```") {
                stripped.to_string()
            } else {
                response
            };
            // TODO: this should be debug log
            info!("[Response] {}", response);
            self.messages.push(("model", response.clone()));
            let response = self.parse_response(&response);
            let mut user_response = String::new();
            for part in response {
                match part {
                    TaskPart::Run { command } => {
                        use std::io::{BufRead, BufReader};
                        use std::process::{Command, Stdio};

                        let mut child = Command::new("sh")
                            .arg("-c")
                            .arg(command.clone())
                            .stdin(Stdio::inherit())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()?;

                        let stdout = child.stdout.take().unwrap();
                        let stderr = child.stderr.take().unwrap();

                        let output = String::new();
                        let error = String::new();

                        // Read stdout
                        let stdout_handle = std::thread::spawn({
                            let mut out = output.clone();
                            move || {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines() {
                                    let line = line.unwrap();
                                    println!("{}", line);
                                    out.push_str(&line);
                                    out.push('\n');
                                }
                                out
                            }
                        });

                        // Read stderr
                        let stderr_handle = std::thread::spawn({
                            let mut err = error.clone();
                            move || {
                                let reader = BufReader::new(stderr);
                                for line in reader.lines() {
                                    let line = line.unwrap();
                                    eprintln!("{}", line);
                                    err.push_str(&line);
                                    err.push('\n');
                                }
                                err
                            }
                        });

                        let status = child.wait()?;
                        let output = stdout_handle.join().unwrap();
                        let error = stderr_handle.join().unwrap();

                        user_response.push_str(&format!(
                            "The output of the command `{}` is:\n```\n{}\n```",
                            command, output
                        ));
                        user_response.push_str(&format!(
                            "The error of the command `{}` is:\n```\n{}\n```",
                            command, error
                        ));
                        user_response.push_str(&format!(
                            "The status of the command `{}` is:\n```\n{}\n```",
                            command, status
                        ));
                    }
                    TaskPart::Message { text } => {
                        info!("[Message] {}", text);
                        println!("Bot: {}", text);
                    }
                    TaskPart::Reason { text } => {
                        info!("[Reason] {}", text);
                    }
                    TaskPart::FileRead { path } => {
                        info!("[File read] {}", path);
                        if let Ok(content) = std::fs::read_to_string(path.clone()) {
                            let content = content
                                .lines()
                                .enumerate()
                                .map(|(line_number, line)| format!("{}: {}", line_number + 1, line))
                                .collect::<Vec<String>>()
                                .join("\n");
                            info!("[Content] {}", content);
                            user_response.push_str(&format!(
                                "The content of the file `{}` is:\n```\n{}\n```",
                                path.clone(),
                                content
                            ));
                        } else {
                            user_response
                                .push_str(&format!("The file `{}` does not exist.", path.clone()));
                        }
                    }
                    TaskPart::FileWriteAdd {
                        path,
                        content,
                        start,
                    } => {
                        info!("[FileWriteAdd] {} at {}", path, start);
                        info!("[Content] {}", content);
                        if let Ok(file_content) = std::fs::read_to_string(path.clone()) {
                            let mut lines = file_content.lines().collect::<Vec<_>>();
                            lines.insert(start as usize, &content);
                            let content = lines.join("\n");
                            std::fs::write(path.clone(), content).unwrap();
                        } else {
                            std::fs::write(path.clone(), content.clone()).unwrap();
                        }
                    }
                    TaskPart::FileWriteReplace {
                        path,
                        content,
                        start,
                        end,
                    } => {
                        info!("[FileWriteReplace] {} at {} to {}", path, start, end);
                        info!("[Content] {}", content);
                        let file_content = std::fs::read_to_string(path.clone()).unwrap();
                        let mut lines = file_content.lines().collect::<Vec<_>>();
                        let start = if start == 0 { 0 } else { start as usize - 1 };
                        let end = end as usize;
                        lines.drain(start..end);
                        lines.insert(start, &content);
                        let content = lines.join("\n");
                        std::fs::write(path.clone(), content).unwrap();
                    }
                    TaskPart::End { reason } => {
                        info!("[End] {}", reason);
                        should_exit = true;
                    }
                }
            }
            if !user_response.is_empty() {
                self.messages.push(("user", user_response));
            } else {
                self.messages.push(("user", "Please continue, use any command/tags whatever you need to. Choose the sanest option.
                You might be missing something. Ensure you have the info about the environment that you need".to_string()));
            }
        }
        Ok(())
    }

    fn parse_response(&self, response: &str) -> Vec<TaskPart> {
        let response = response.replace("```json", "").replace("```", "");
        serde_json::from_str::<Vec<TaskPart>>(&response).unwrap()
    }

    fn response_format(&self) -> String {
        "

        The user is another agent that forwards you the task.
        You need to complete the task provided by the user.
        To do so, you may need to run commands, read files, write to files, or send messages to the user.
        The user will respond with the result of the commands you run, the content of the files you read, and the messages you send
        if they deem it necessary to do so.
        While writing to files, use line numbers that you see when you read the file.
        Also, when writing files, every message related to writing could change the line numbers, so the next message should
        ensure that the line numbers are correct and updated based on the content written in previous messages.

        Respond in the following format meant for fash.
        type Message = Run | Message | Reason | FileWriteAdd | FileWriteReplace | FileRead | End;
        // Run a command
        type Run = {{
            type: 'run',
            command: String,
        }};
        // Send a message to the user
        type Message = {{
            type: 'message',
            text: String,
        }};
        // Explain the reason for the action
        type Reason = {{
            type: 'reason',
            text: String,
        }};
        // Add to a file
        type FileWriteAdd = {{
            type: 'file-write-add',
            path: String,
            start: usize,
            content: String,
        }};
        // Replace in a file
        type FileWriteReplace = {{
            type: 'file-write-replace',
            path: String,
            start: usize,
            end: usize,
            content: String,
        }};
        // Read a file - will return the content of the file along with line numbers
        type FileRead = {{
            type: 'file-read',
            path: String,
        }};
        // End the session
        type End = {{
            type: 'end',
            reason: String,
        }};

        For example, to say hello to the user, and write first ten lines from the output of the command `ls -l` to a file, and read the file, you can use:
        [
            {{
                type: 'message',
                text: 'Hello!',
            }},
            {{
                type: 'run',
                command: 'ls -l | head -n 10 > hello.txt',
                output: 'Hello!',
            }},
            {{
                type: 'file-read',
                path: 'hello.txt',
            }},
            {{
                type: 'end',
                reason: 'Task complete',
            }},
        ]

        Start your response with reasoning within the reason field.
        If it's a task, you need to follow the following steps for reasoning:
        1. Analyze the task and the user's request or analyze how you want to overcome the limitations that you might have listed down.
            1.1 In case of doubts, rethink the task and what the user implies by their request. You can assume the user is smart and knows what they are talking about.
            1.2 Think of what you're about to do and whether that makes sense from the user's perspective.
                The first thing you can always try in case of doubts is to see what files are there that you can read or run arbitrary commands to gain information.
        2. Break down the task into smaller steps. Ensure you think about the task's details like what line numbers you need to work with, what command to run, etc
        3. Pick up the next step that's not yet completed and decide what you need to do to complete it.
        4. Think of any limitations in the task and in the current step
        5. List ways to overcome the limitations

        One by one, you should do what you said you will do, and showing a message to the 
        user wherever you feel the need.
        Be very pro-active and take initiative. You need to be the one who drives the task forward.
        This means you need to get the information you need by yourself and do what you need to do.
        You will be run in a sandbox environment so feel free to do whatever you need to do.
        You may use any of the tags mentioned above multiple times in your response.
        Take as much information as you can and provide the best possible response so that it is
        useful to the user.

        When the task is complete, use the message field to send a message to the user with
        the result of the task. Then a reason field.
        Then use the end field to end the session if the task is complete.
        If the task is not complete for whatever reason, you need to continue working on the task and
        reason about what you need to do next.
        
        Before you end the session, you need to show a reason for ending the session in the
        reason field. If in your reasoning, you realise that the task is incomplete, you need to skip
        ending the session and instead plan for the next step and use any tags you need to get you 
        closer to completing the task.
        Ensure you collect all the information you need before starting. Ensure perfection.
        Understand the world and the words will flow.

        You can check the README.md file of a project to understand a project.

        There is literally nothing you can't do.
        ".to_string()
    }
}

impl std::fmt::Debug for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Agent>")
    }
}

