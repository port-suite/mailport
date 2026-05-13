use tokio::net::TcpStream;

use crate::smtp;

pub struct Helo<'a> {
    stream: &'a TcpStream,
    host: String,
}

impl Helo<'_> {
    pub fn new(stream: &TcpStream, host: String) -> Helo<'_> {
        Helo { stream, host }
    }

    pub async fn execute(&self, session: &mut smtp::Session) -> Result<(), anyhow::Error> {
        // Save to session
        // Send response
        println!(
            "Got HELO command with payload '{}'; Sending 250 Hello, {}",
            self.host, self.host
        );
        session.helo = Some(self.host.clone());
        self.stream.writable().await?;
        let msg_string = format!("250 Hello, {}\r\n", self.host);
        self.stream.try_write(msg_string.as_bytes())?;
        Ok(())
    }
}
