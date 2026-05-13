use tokio::net::TcpStream;

use crate::smtp;

pub struct MailFrom<'a> {
    stream: &'a TcpStream,
    sender: String,
}

impl MailFrom<'_> {
    pub fn new(stream: &TcpStream, sender: String) -> MailFrom<'_> {
        MailFrom { stream, sender }
    }

    pub async fn execute(&self, session: &mut smtp::Session) -> Result<(), anyhow::Error> {
        self.stream.writable().await?;
        if session.helo.is_none() {
            self.stream.try_write(b"503 Bad sequence of commands\r\n")?;
            return Ok(());
        }
        if self.sender.chars().nth(0).unwrap() != '<'
            || self.sender.chars().nth(self.sender.len() - 1).unwrap() != '>'
        {
            self.stream.try_write(b"501 Syntax error\r\n")?;
            return Ok(());
        }
        session.mail_from = Some(self.sender.clone());
        self.stream.try_write(b"250 OK\r\n")?;
        Ok(())
    }
}
