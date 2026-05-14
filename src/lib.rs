use std::ops::IndexMut;

use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

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
        let (reader, writer) = socket.into_split();
        let mut reader = BufReader::new(reader);
        let mut writer = BufWriter::new(writer);
        println!("Connection from: {}", addr);

        // match socket.writable().await {
        //     Ok(_) => (),
        //     Err(err) => {
        //         eprintln!("{err}");
        //         std::process::exit(-1);
        //     }
        // }
        let _ = writer.write_all(b"220 www.mailport.com\r\n").await;
        let _ = writer.flush().await;
        // match socket.try_write(b"220 www.mailport.com\r\n") {
        //     Ok(_) => (),
        //     Err(err) => {
        //         eprintln!("{err}");
        //         std::process::exit(-1);
        //     }
        // };
        tokio::spawn(async move {
            let mut session = smtp::Session::new();
            loop {
                let cmd = match read_command(&mut reader).await {
                    Ok(cmd) => cmd,
                    Err(err) => {
                        eprintln!("{err}");
                        break;
                    }
                };
                let should_quit = match cmd.do_command(&mut reader, &mut writer, &mut session).await
                {
                    Ok(res) => !res,
                    Err(err) => {
                        eprintln!("{err}");
                        break;
                    }
                };
                if should_quit {
                    break;
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct ParsedSMTP {
    command: smtp::SmtpCommand,
    payload: String,
}

impl ParsedSMTP {
    fn new(command: smtp::SmtpCommand, payload: String) -> Self {
        Self { command, payload }
    }

    #[allow(unused)]
    async fn do_command(
        &self,
        reader: &mut BufReader<OwnedReadHalf>,
        writer: &mut BufWriter<OwnedWriteHalf>,
        session: &mut smtp::Session,
    ) -> Result<bool, anyhow::Error> {
        match self.command {
            smtp::SmtpCommand::Quit => {
                let mut quit_cmd = smtp::quit::Quit::new(writer);
                quit_cmd.execute().await?;
                return Ok(false);
            }
            smtp::SmtpCommand::Noop => {
                let mut noop_cmd = smtp::noop::Noop::new(writer);
                noop_cmd.execute().await?;
            }
            smtp::SmtpCommand::Helo => {
                let mut helo_cmd = smtp::helo::Helo::new(writer, self.payload.clone());
                helo_cmd.execute(session).await?;
                println!("{session:?}")
            }
            smtp::SmtpCommand::MailFrom => {
                let mut mail_from_cmd =
                    smtp::mail_from::MailFrom::new(writer, self.payload.clone());
                mail_from_cmd.execute(session).await?;
                println!("{session:?}")
            }
            smtp::SmtpCommand::RcptTo => {
                let mut rcpt_to_cmd = smtp::rcpt_to::RcptTo::new(writer, self.payload.clone());
                rcpt_to_cmd.execute(session).await?;
                println!("{session:?}");
            }
            smtp::SmtpCommand::Data => {
                let mut data_cmd = smtp::data::Data::new(reader, writer);
                data_cmd.execute(session).await?;
                println!("{session:?}");
            }
            smtp::SmtpCommand::Unknown => {
                let _ = writer.write_all(b"500 Unrecognized command\r\n").await;
                let _ = writer.flush().await;
            } // _ => {
              //     println!("command not reconized: {}", self.command);
              // }
        };
        Ok(true)
    }
}

async fn read_command(stream: &mut BufReader<OwnedReadHalf>) -> Result<ParsedSMTP, anyhow::Error> {
    // stream.readable().await?;
    let mut buf = String::new();
    #[allow(unused)]
    let mut bytes_num: usize = 0;
    let _ = stream.read_line(&mut buf).await?;
    // {
    //      Ok(n) => {
    //          bytes_num = n;
    //          break;
    //      }
    //      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
    //          stream.readable().await?;
    //      }
    //      Err(err) => {
    //          return Err(anyhow!(err));
    //      }
    //  };
    // let mut trimmed_buff = &buf[..bytes_num];
    // if trimmed_buff[trimmed_buff.len() - 2] != 0x0d || trimmed_buff[trimmed_buff.len() - 1] != 0x0a
    // {
    //     return Err(anyhow!("Request with incorrect format"));
    // }

    // trimmed_buff = &trimmed_buff[..trimmed_buff.len() - 2];
    // let line: String = String::from_utf8(trimmed_buff.to_vec())?;
    let line = buf.trim_end_matches(&['\r', '\n'][..]);
    if line.is_empty() {
        return Ok(ParsedSMTP {
            command: smtp::SmtpCommand::Unknown,
            payload: String::new(),
        });
    }
    let mut command: Option<ParsedSMTP> = None;
    let possible_commands = vec!["HELO", "MAIL FROM:", "RCPT TO:", "DATA", "QUIT", "NOOP"];
    for cmd in possible_commands {
        let re = Regex::new(format!("({})\\s?(.*)", cmd).as_str()).unwrap();
        let Some(matches) = re.captures(line) else {
            continue;
        };
        command = Some(ParsedSMTP::new(
            smtp::SmtpCommand::from_string(matches[1].to_string()),
            matches[2].to_string(),
        ));
    }
    if command.is_none() {
        return Ok(ParsedSMTP {
            command: smtp::SmtpCommand::Unknown,
            payload: String::new(),
        });
    }
    Ok(command.unwrap())
}
