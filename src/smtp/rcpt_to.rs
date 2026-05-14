use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::tcp::OwnedWriteHalf,
};

use crate::smtp;

pub struct RcptTo<'a> {
    stream: &'a mut BufWriter<OwnedWriteHalf>,
    recipient: String,
}

impl RcptTo<'_> {
    pub fn new(stream: &mut BufWriter<OwnedWriteHalf>, recipient: String) -> RcptTo<'_> {
        RcptTo { stream, recipient }
    }

    pub async fn execute(&mut self, session: &mut smtp::Session) -> Result<(), anyhow::Error> {
        if session.mail_from.is_none() || session.helo.is_none() {
            self.stream
                .write_all(b"503 Bad sequence of commands\r\n")
                .await?;
            self.stream.flush().await?;
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
            self.stream.write_all(b"501 Syntax error\r\n").await?;
            self.stream.flush().await?;

            return Ok(());
        }
        session.rcpt_to.push(String::from(&self.recipient));
        self.stream.write_all(b"250 OK\r\n").await?;
        self.stream.flush().await?;
        Ok(())
    }
}
