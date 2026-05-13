use anyhow::anyhow;
use regex::Regex;

mod smtp;

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

        match socket.readable().await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(-1);
            }
        };
        tokio::spawn(async move {
            let mut session = smtp::Session::new();
            loop {
                let cmd = match read_command(&socket).await {
                    Ok(cmd) => cmd,
                    Err(err) => {
                        eprintln!("{err}");
                        break;
                    }
                };
                let res = match cmd.do_command(&socket, &mut session).await {
                    Ok(res) => res,
                    Err(err) => {
                        eprintln!("{err}");
                        break;
                    }
                };
                if !res {
                    break;
                }
            }
        });
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct ParsedSMTP {
    command: smtp::SmtpCommand,
    payload: String,
}

impl ParsedSMTP {
    fn new(command: smtp::SmtpCommand, payload: String) -> Self {
        Self { command, payload }
    }

    async fn do_command(
        &self,
        stream: &tokio::net::TcpStream,
        session: &mut smtp::Session,
    ) -> Result<bool, anyhow::Error> {
        match self.command {
            smtp::SmtpCommand::Quit => {
                let quit_cmd = smtp::quit::Quit::new(stream);
                quit_cmd.execute().await?;
                return Ok(false);
            }
            smtp::SmtpCommand::Noop => {
                let noop_cmd = smtp::noop::Noop::new(stream);
                noop_cmd.execute().await?;
            }
            smtp::SmtpCommand::Helo => {
                let helo_cmd = smtp::helo::Helo::new(stream, self.payload.clone());
                helo_cmd.execute(session).await?;
                println!("{session:?}")
            }
            _ => {
                println!("command not reconized: {}", self.command);
            }
        };
        Ok(true)
    }
}

async fn read_command(stream: &tokio::net::TcpStream) -> Result<ParsedSMTP, anyhow::Error> {
    stream.readable().await?;
    let mut buf = [0u8; 512];
    #[allow(unused)]
    let mut bytes_num: usize = 0;
    loop {
        match stream.try_read(&mut buf) {
            Ok(n) => {
                bytes_num = n;
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                stream.readable().await?;
            }
            Err(err) => {
                return Err(anyhow!(err));
            }
        };
    }
    let mut trimmed_buff = &buf[..bytes_num];
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
            smtp::SmtpCommand::from_string(matches[1].to_string()),
            matches[2].to_string(),
        ));
    }
    if command.is_none() {
        return Err(anyhow!("Could not read any command"));
    }
    Ok(command.unwrap())
}
