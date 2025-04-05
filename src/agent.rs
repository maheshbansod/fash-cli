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
            I am ready to start the task.
            </l-reason>
            <l-message>
            yes
            </l-message>
            <l-end></l-end>".to_string()),
            ("user", task.to_string()),
        ];
        let system_prompt = format!("Your name is fash.\n{}\n\n{}", system_prompt, response_format);
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
                    TaskPart::Reason(_reason) => {
                        // println!("[Reason] {}", reason);
                    }
                    TaskPart::FileRead(file_read) => {
                        let content = std::fs::read_to_string(file_read.path).unwrap();
                        println!("[File read] {}", file_read.path);
                        let content = content.lines().enumerate().map(|(line_number, line)| format!("{}: {}", line_number + 1, line)).collect::<Vec<String>>().join("\n");
                        user_response.push_str(&format!("The content of the file `{}` is:\n```\n{}\n```", file_read.path, content));
                    }
                    // TaskPart::FileWriteAdd(file_write_add) => {
                    //     let content = std::fs::read_to_string(file_write_add.path).unwrap();
                    //     println!("[FileWriteAdd] {}", content);
                    // }
                    // TaskPart::FileWriteReplace(file_write_replace) => {
                    //     let content = std::fs::read_to_string(file_write_replace.path).unwrap();
                    //     println!("[FileWriteReplace] {}", content);
                    // }
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
                self.messages.push(("user", "Please continue".to_string()));
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

        You need to complete the task provided by the user.
        To do so, you may need to run commands, read files, write to files, or send messages to the user.
        The user will respond with the result of the commands you run, the content of the files you read, and the messages you send
        if they deem it necessary to do so.

        Respond in the following special format meant for fash. It's not XML or HTML or Markdown.
        You can use only the following tags at the top level:
        l-run: Run a command
        l-message: Send a message
        l-reason: Explain the reason for the action
        l-file-write-add: Add to a file
        l-file-write-replace: Replace in a file
        l-file-read: Read a file - content is returned with line numbers
        l-end: End the session. You can choose to end the session once you have completed the task. The user may still choose to respond and start a new task if they want.

        l-file-write-add allows a few inner tags:
           l-fw-path: The file path
           l-fw-start: The line number before which to start adding on. (use 0 for start of file)
           l-fw-content: The content to add

        l-file-write-replace allows a few inner tags:
           l-fw-path: The file path
           l-fw-start: The line number before which to start replacing. (use 0 for start of file)
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
        In your reasoning, mention the steps you will take to complete the task.
        In your reasoning, in the last step, review your reasoning and decide on what you'd like
        to do so that you can complete the task fully.
        Every time you communicate with the user, you should start with the detailed reasoning
        with the reasoning steps so that you can complete the task fully. Every time you
        need to contemplate whether the task is complete. Then after the reasoning, you should
        show a message to the user about what you will do next.
        And one by one, you should do what you said you will do, and showing a message to the 
        user wherever you feel the need.
        You may use any of the tags mentioned above multiple times in your response.


        ")
    }
} 