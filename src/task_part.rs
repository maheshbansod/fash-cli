use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TaskPart {
    Run {
        command: String,
    },
    Message {
        text: String,
    },
    Reason {
        text: String,
    },
    FileWriteAdd {
        path: String,
        content: String,
        start: u32,
    },
    FileWriteReplace {
        path: String,
        content: String,
        start: u32,
        end: u32,
    },
    FileRead {
        path: String,
    },
    End {
        reason: String,
    },
}
