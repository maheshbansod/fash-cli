use std::error::Error;
use tokio::process::Command;

use crate::config::Config;
use crate::gemini::GeminiClient;
use crate::parser::TaskPart;

type Message = (&'static str, String);

pub struct Agent {
    config: Config,
    client: GeminiClient,
    messages: Vec<Message>,
}

impl Agent {
    pub fn new(api_key: String) -> Self {
        let config = Config::load();
        let client = GeminiClient::new(api_key);
        
        Self {
            config,
            client,
            messages: vec![],
        }
    }
    
    pub async fn run(&mut self, task: &str) -> Result<(), Box<dyn Error>> {
        let system_prompt = self.config.get_system_prompt();
        let response_format = self.response_format();
        self.messages = vec![
            ("user","Respond with yes if you are ready.".to_string()),
            ("model", "
            <l-reason>
            It's a simple question. I am ready to start the task.
            </l-reason>
            <l-message>
            yes
            </l-message>
            <l-reason>
            The user asked a simple question and I have fully answered it so I can end the session.
            </l-reason>
            <l-end></l-end>".to_string()),
            ("user", format!("The task is: {}", task)),
        ];
        let system_prompt = format!("Your name is fash. You are an autonomous agent that will be run ina terminal with very limited user interaction.\n{}\n\n{}", system_prompt, response_format);
        let mut should_exit = false;
        while !should_exit {
            let response = self.client.generate_content(&self.messages, &system_prompt).await?;
            // println!("[Response] {}", response);
            // remove the first line of response if it starts with ``` and also remove the last ``` in the response
            let response = if response.starts_with("```") {
                response[1..].to_string()
            } else {
                response
            };
            let response = if response.ends_with("```") {
                response[..response.len() - 3].to_string()
            } else {
                response
            };
            println!("[Response] {}", response);
            self.messages.push(("model", response.clone()));
            let response = self.parse_response(&response);
            // println!("[Parsed Response] {:?}", response);
            let mut user_response = String::new();
            for part in response {
                match part {
                    TaskPart::Run(command) => {
                        let output = Command::new("sh").arg("-c").arg(command).output().await?;
                        let command_output = String::from_utf8(output.stdout).unwrap();
                        println!("[Run] {}", command);
                        user_response.push_str(&format!("The output of the command `{}` is:\n```\n{}\n```", command, command_output));
                    }
                    TaskPart::Message(message) => {
                        println!("[Message] {}", message);
                    }
                    TaskPart::Reason(reason) => {
                        println!("[Reason] {}", reason);
                    }
                    TaskPart::FileRead(file_read) => {
                        println!("[File read] {}", file_read.path);
                        if let Ok(content) = std::fs::read_to_string(file_read.path) {
                            let content = content.lines().enumerate().map(|(line_number, line)| format!("{}: {}", line_number + 1, line)).collect::<Vec<String>>().join("\n");
                            println!("[Content] {}", content);
                            user_response.push_str(&format!("The content of the file `{}` is:\n```\n{}\n```", file_read.path, content));
                        } else {
                            user_response.push_str(&format!("The file `{}` does not exist.", file_read.path));
                        }
                    }
                    TaskPart::FileWriteAdd(file_write_add) => {
                        println!("[FileWriteAdd] {}", file_write_add.path);
                        if let Ok(content) = std::fs::read_to_string(file_write_add.path) {
                            let mut lines = content.lines().collect::<Vec<_>>();
                            lines.insert(file_write_add.start as usize, file_write_add.content);
                            let content = lines.join("\n");
                            std::fs::write(file_write_add.path, content).unwrap();
                        } else {
                            std::fs::write(file_write_add.path, file_write_add.content).unwrap();
                        }
                    }
                    TaskPart::FileWriteReplace(file_write_replace) => {
                        println!("[FileWriteReplace] {}", file_write_replace.path);
                        println!("[Content] {}", file_write_replace.content);
                        println!("[Start] {}", file_write_replace.start);
                        println!("[End] {}", file_write_replace.end);
                        let content = std::fs::read_to_string(file_write_replace.path).unwrap();
                        let mut lines = content.lines().collect::<Vec<_>>();
                        let start = file_write_replace.start as usize;
                        let end = file_write_replace.end as usize;
                        lines.drain(start..end);
                        lines.insert(start, file_write_replace.content);
                        let content = lines.join("\n");
                        std::fs::write(file_write_replace.path, content).unwrap();
                    }
                    TaskPart::End => {
                        println!("[End] Ending session");
                        should_exit = true;
                    },
                    _ => todo!()
                }
            }
            if !user_response.is_empty() {
                self.messages.push(("user", user_response));
            } else {
                self.messages.push(("user", "Please continue, use any command/tags whatever you need to".to_string()));
            }
        }
        Ok(())
    }

    fn parse_response<'a>(&self, response: &'a str) -> Vec<TaskPart<'a>> {
        let mut parser = crate::parser::Parser::new(response);
        parser.parse()
    }

    fn response_format(&self) -> String {
        format!("

        The user is another agent that forwards you the task.
        You need to complete the task provided by the user.
        To do so, you may need to run commands, read files, write to files, or send messages to the user.
        The user will respond with the result of the commands you run, the content of the files you read, and the messages you send
        if they deem it necessary to do so.

        Respond in the following special format meant for fash. It's not XML or HTML or Markdown.
        You can use only the following tags at the top level:
        l-run: Run a command
        l-message: Send a message to the user - this is the only way to communicate with the user
        l-reason: Explain the reason for the action
        l-file-write-add: Add to a file
        l-file-write-replace: Replace in a file
        l-file-read: Read a file - content is returned with line numbers. You should read files often to understand the world.
        l-end: End the session. You can choose to end the session once you have completed the task. The user may still choose to respond and start a new task if they want.

        l-file-write-add allows a few inner tags:
           l-fw-path: The file path
           l-fw-start: The line number before which to start adding on. (use 0 to insert at the start of file)
           l-fw-content: The content to add

        l-file-write-replace allows a few inner tags:
           l-fw-path: The file path
           l-fw-start: The line number before which to start replacing. (use 0 to replace at the start of file)
           l-fw-end: The line number on which to end replacing.
           l-fw-content: The content to write and replace the existing content with.

        l-file-read allows a few inner tags:
           l-fr-path: The file path

        For example, to say hello to the user, and echo hello in a file, and read the file, you can use:
        <l-message>
        Hello!
        </l-message>
        <l-file-write-add>
           <l-fw-path>hello.txt</l-fw-path>
           <l-fw-content>Hello!</l-fw-content>
        </l-file-write-add>
        <l-file-read>
           <l-fw-path>hello.txt</l-fw-path>
        </l-file-read>

        Follow this format if you want to be taken seriously.
        Start your response with reasoning within the reason tag.
        If it's a task, you need to follow the following steps for reasoning:
        1. Analyze the task and the user's request or analyze how you want to overcome the limitations that you might have listed down.
        2. Break down the task into smaller steps
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

        When the task is complete, use the l-message tag to send a message to the user with
        the result of the task. Then a reason tag.
        Then use the l-end tag to end the session if the task is complete.
        If the task is not complete for whatever reason, you need to continue working on the task and
        reason about what you need to do next.
        
        Before you end the session, you need to show a reason for ending the session in the
        reason tag. If in your reasoning, you realise that the task is incomplete, you need to skip
        ending the session and instead plan for the next step and use any tags you need to get you 
        closer to completing the task.
        Ensure you collect all the information you need before starting. Ensure perfection.
        Understand the world and the words will flow.

        You can check the README.md file of a project to understand a project.

        There is literally nothing you can't do.
        ")
    }
} 