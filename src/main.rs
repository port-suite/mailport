use core::fmt;

use anyhow::anyhow;
use regex::Regex;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:8010").await {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Could not start TCP server");
            std::process::exit(-1);
        }
    };
    println!("Listening on :8010");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("Connection from: {}", addr);
        let cmd = match read_command(&socket).await {
            Ok(cmd) => cmd,
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(-1);
            }
        };
        println!("{cmd:?}");
    }
}

#[allow(unused)]
#[derive(Debug)]
struct SmtpCommand {
    command: String,
    payload: String,
}

impl SmtpCommand {
    fn new(command: String, payload: String) -> Self {
        Self { command, payload }
    }
}

async fn read_command(stream: &tokio::net::TcpStream) -> Result<SmtpCommand, anyhow::Error> {
    let mut buf = [0u8; 512];
    let _ = stream.readable().await;
    let n = match stream.try_read(&mut buf) {
        Ok(n) => n,
        Err(err) => {
            println!("could not read");
            return Err(anyhow!(err));
        }
    };
    let mut trimmed_buff = &buf[..n];
    if trimmed_buff[trimmed_buff.len() - 2] != 0x0d || trimmed_buff[trimmed_buff.len() - 1] != 0x0a
    {
        return Err(anyhow!("Request with incorrect format"));
    }
    trimmed_buff = &trimmed_buff[..trimmed_buff.len() - 2];
    let line: String = String::from_utf8(trimmed_buff.to_vec())?;
    let mut command: Option<SmtpCommand> = None;
    let possible_commands = vec![
        "HELO", "EHLO", // "MAIL FROM",
        // "RCPT TO",
        // "DATA",
        "QUIT", "NOOP",
    ];
    for cmd in possible_commands {
        let re = Regex::new(format!("({})\\s?(.*)", cmd).as_str()).unwrap();
        let Some(matches) = re.captures(&line) else {
            continue;
        };
        command = Some(SmtpCommand::new(
            String::from(&matches[1].to_string()),
            String::from(&matches[2].to_string()),
        ));
    }
    if command.is_none() {
        return Err(anyhow!("Could not read any command"));
    }
    Ok(command.unwrap())
}
