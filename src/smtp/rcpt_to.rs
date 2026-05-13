use tokio::net::TcpStream;

use crate::smtp;

pub struct RcptTo<'a> {
    stream: &'a TcpStream,
    recipient: String,
}

impl RcptTo<'_> {
    pub fn new(stream: &TcpStream, recipient: String) -> RcptTo<'_> {
        RcptTo { stream, recipient }
    }

    pub async fn execute(&self, session: &mut smtp::Session) -> Result<(), anyhow::Error> {
        self.stream.writable().await?;
        if session.mail_from.is_none() || session.helo.is_none() {
            self.stream.try_write(b"503 Bad sequence of commands\r\n")?;
            return Ok(());
        }
        if self.recipient.chars().nth(0).unwrap() != '<'
            || self
                .recipient
                .chars()
                .nth(self.recipient.len() - 1)
                .unwrap()
                != '>'
        {
            self.stream.try_write(b"501 Syntax error\r\n")?;
            return Ok(());
        }
        session.rcpt_to.push(String::from(&self.recipient));
        self.stream.try_write(b"250 OK\r\n")?;
        Ok(())
    }
}
