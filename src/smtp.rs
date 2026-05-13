use std::fmt::Display;

pub mod helo;
pub mod noop;
pub mod quit;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    helo: Option<String>,
    sender: Option<String>,
    receivers: Vec<String>,
    data: Option<String>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            helo: None,
            sender: None,
            receivers: vec![],
            data: None,
        }
    }
}

#[derive(Debug)]
pub enum SmtpCommand {
    Quit,
    Noop,
    Helo,
    Unknown,
}

impl SmtpCommand {
    pub fn from_string(cmd_string: String) -> SmtpCommand {
        match cmd_string.as_str() {
            "QUIT" => Self::Quit,
            "NOOP" => Self::Noop,
            "HELO" => Self::Helo,
            _ => Self::Unknown,
        }
    }
}

impl Display for SmtpCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quit => write!(f, "quit command"),
            Self::Noop => write!(f, "noop command"),
            Self::Helo => write!(f, "helo command"),
            Self::Unknown => write!(f, "unknown command"),
        }
    }
}
