use anyhow::anyhow;
use regex::Regex;

mod commands;

pub async fn run() {
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
        tokio::spawn(async move {
            cmd.do_command(socket).await;
        });
    }
}

#[allow(unused)]
#[derive(Debug)]
struct ParsedSMTP {
    command: String,
    payload: String,
}

impl ParsedSMTP {
    fn new(command: String, payload: String) -> Self {
        Self { command, payload }
    }

    async fn do_command(&self, stream: tokio::net::TcpStream) {
        match self.command.clone().as_str() {
            "QUIT" => {
                let quit_cmd = commands::quit::Quit::new(&stream);
                let _ = quit_cmd.execute().await;
                drop(stream);
            }
            _ => {
                println!("command not reconized: {}", self.command);
            }
        };
    }
}

async fn read_command(stream: &tokio::net::TcpStream) -> Result<ParsedSMTP, anyhow::Error> {
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
    let mut command: Option<ParsedSMTP> = None;
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
        command = Some(ParsedSMTP::new(
            matches[1].to_string(),
            matches[2].to_string(),
        ));
    }
    if command.is_none() {
        return Err(anyhow!("Could not read any command"));
    }
    Ok(command.unwrap())
}
