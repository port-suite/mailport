use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::tcp::OwnedWriteHalf,
};

use crate::smtp;

pub struct Helo<'a> {
    stream: &'a mut BufWriter<OwnedWriteHalf>,
    host: String,
}

impl Helo<'_> {
    pub fn new(stream: &mut BufWriter<OwnedWriteHalf>, host: String) -> Helo<'_> {
        Helo { stream, host }
    }

    pub async fn execute(&mut self, session: &mut smtp::Session) -> Result<(), anyhow::Error> {
        // TODO: make sure payload is not empty
        // TODO: check payload validity
        println!(
            "Got HELO command with payload '{}'; Sending 250 OK",
            self.host
        );
        session.helo = Some(self.host.clone());
        let msg = b"250 OK\r\n";
        self.stream.write_all(msg).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
