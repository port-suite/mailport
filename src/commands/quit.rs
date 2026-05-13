pub struct Quit<'a> {
    stream: &'a tokio::net::TcpStream,
}

impl Quit<'_> {
    pub fn new(stream: &tokio::net::TcpStream) -> Quit<'_> {
        Quit { stream }
    }

    pub async fn execute(&self) -> Result<(), anyhow::Error> {
        self.stream.writable().await?;
        let msg = b"221 Closed\r\n";
        let n = &self.stream.try_write(msg)?;
        println!("Wrote {} bytes and closed the stream", n);
        Ok(())
    }
}
