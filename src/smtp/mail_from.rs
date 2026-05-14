use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::tcp::OwnedWriteHalf,
};

use crate::smtp;

pub struct MailFrom<'a> {
    stream: &'a mut BufWriter<OwnedWriteHalf>,
    sender: String,
}

impl MailFrom<'_> {
    pub fn new(stream: &mut BufWriter<OwnedWriteHalf>, sender: String) -> MailFrom<'_> {
        MailFrom { stream, sender }
    }

    pub async fn execute(&mut self, session: &mut smtp::Session) -> Result<(), anyhow::Error> {
        if session.helo.is_none() {
            self.stream
                .write_all(b"503 Bad sequence of commands\r\n")
                .await?;
            self.stream.flush().await?;
            return Ok(());
        }
        if self.sender.chars().nth(0).unwrap() != '<'
            || self.sender.chars().nth(self.sender.len() - 1).unwrap() != '>'
        {
            self.stream.write_all(b"501 Syntax error\r\n").await?;
            self.stream.flush().await?;
            return Ok(());
        }
        session.mail_from = Some(self.sender.clone());
        self.stream.write_all(b"250 OK\r\n").await?;
        self.stream.flush().await?;
        Ok(())
    }
}
