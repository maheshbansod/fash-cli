use std::str::CharIndices;


pub struct Parser<'a> {
    full_text: &'a str,
    char_iterator: CharIndices<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(response: &'a str) -> Self {
        Self { full_text: response, char_iterator: response.char_indices() }
    }

    pub fn parse(&mut self) -> Vec<TaskPart<'a>> {
        let mut tags = vec![];
        while let Some(tag) = self.parse_tag() {
            tags.push(tag);
        }
        tags
    }

    fn consume_whitespace(&mut self) {
        let mut c_clone = self.char_iterator.clone();
        while let Some((_, c)) = c_clone.next() {
            if !c.is_whitespace() {
                break;
            }
            self.char_iterator.next();
        }
    }

    fn parse_tag(&mut self) -> Option<TaskPart<'a>> {
        self.consume_whitespace();
        self.parse_until(|c| c == '<');
        self.char_iterator.next();
        let tag_name = self.parse_tag_name();
        if tag_name.is_empty() {
            return None;
        }
        match tag_name {
            "l-message" => {
                let message = self.parse_until_end_tag("l-message");
                return Some(TaskPart::Message(message))
            }
            "l-run" => {
                let run = self.parse_until_end_tag("l-run");
                return Some(TaskPart::Run(run))
            }
            "l-reason" => {
                let reason = self.parse_until_end_tag("l-reason");
                return Some(TaskPart::Reason(reason))
            }
            "l-end" => {
                self.parse_until_end_tag("l-end");
                return Some(TaskPart::End)
            }
            "l-file-read" => {
                let mut file_read = FileRead { path: "" };
                loop {
                    self.consume_whitespace();
                    self.parse_until(|c| c == '<');
                    self.char_iterator.next();
                    let tag_name = self.parse_tag_name();
                    if tag_name == "/l-file-read" {
                        break;
                    } else if tag_name == "l-fr-path" {
                        file_read.path = self.parse_until_end_tag("l-fr-path");
                    } else {
                        println!("Unknown tag: {:?}", tag_name);
                    }
                }
                return Some(TaskPart::FileRead(file_read))
            }
            _ => {
                println!("Unknown tag: {:?}", tag_name);
                return None;
            }
        }
    }

    fn parse_tag_name(&mut self) -> &str {
        let tag_name = self.parse_until(|c| c == '>');
        self.char_iterator.next();
        tag_name
    }

    fn parse_until(&mut self, predicate: impl Fn(char) -> bool) -> &'a str {
        let current_index = if let Some((index, _)) = self.char_iterator.clone().next() {
            index
        } else {
            return "";
        };
        let mut end_index = current_index + 1;
        while let Some((_, c)) = self.char_iterator.clone().next() {
            if predicate(c) {
                break;
            }
            end_index = self.char_iterator.next().unwrap().0 + 1;
        }
        &self.full_text[current_index..end_index]
    }

    fn parse_until_end_tag(&mut self, tag_name: &str) -> &'a str {
        let current_index = self.char_iterator.clone().next().unwrap().0;
        loop {
            self.parse_until(|c| c == '<');
            let end_index = self.char_iterator.clone().next().unwrap().0;
            self.char_iterator.next();// consume '<'
            self.char_iterator.next(); // consume '/'
            let next_tag_name = self.parse_until(|c| c == '>');
            self.char_iterator.next(); // consume '>'
            if next_tag_name == tag_name {
                return &self.full_text[current_index..end_index];
            }
        }
    }
    
}

#[derive(Debug)]
pub enum TaskPart<'a> {
    Run(&'a str),
    Message(&'a str),
    Reason(&'a str),
    FileWriteAdd(FileWriteAdd<'a>),
    FileWriteReplace(FileWriteReplace<'a>),
    FileRead(FileRead<'a>),
    End,
}

#[derive(Debug)]
pub struct FileWriteAdd<'a> {
    path: &'a str,
    content: &'a str,
    start: u32,
}

#[derive(Debug)]
pub struct FileWriteReplace<'a> {
    path: &'a str,
    content: &'a str,
    start: u32,
    end: u32,
}

#[derive(Debug)]
pub struct FileRead<'a> {
    pub path: &'a str,
}