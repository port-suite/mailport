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
        // TODO: make sure payload is not empty
        // TODO: check payload validity
        println!(
            "Got HELO command with payload '{}'; Sending 250 OK",
            self.host
        );
        session.helo = Some(self.host.clone());
        self.stream.writable().await?;
        let msg = b"250 OK\r\n";
        self.stream.try_write(msg)?;
        Ok(())
    }
}
