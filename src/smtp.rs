use std::fmt::Display;

pub mod data;
pub mod helo;
pub mod mail_from;
pub mod noop;
pub mod quit;
pub mod rcpt_to;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    helo: Option<String>,
    mail_from: Option<String>,
    rcpt_to: Vec<String>,
    data: Option<String>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            helo: None,
            mail_from: None,
            rcpt_to: vec![],
            data: None,
        }
    }
}

#[derive(Debug)]
pub enum SmtpCommand {
    Quit,
    Noop,
    Helo,
    MailFrom,
    RcptTo,
    Unknown,
}

impl SmtpCommand {
    pub fn from_string(cmd_string: String) -> SmtpCommand {
        match cmd_string.as_str() {
            "QUIT" => Self::Quit,
            "NOOP" => Self::Noop,
            "HELO" => Self::Helo,
            "MAIL FROM:" => Self::MailFrom,
            "RCPT TO:" => Self::RcptTo,
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
            Self::MailFrom => write!(f, "mail from command"),
            Self::RcptTo => write!(f, "rcpt to command"),
            Self::Unknown => write!(f, "unknown command"),
        }
    }
}
