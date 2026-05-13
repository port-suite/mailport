use tokio::net::TcpStream;

pub struct Noop<'a> {
    stream: &'a TcpStream,
}

impl Noop<'_> {
    pub fn new(stream: &TcpStream) -> Noop<'_> {
        Noop { stream }
    }

    pub async fn execute(&self) -> Result<(), anyhow::Error> {
        println!("Got NOOP command; Sending 250 OK");
        self.stream.writable().await?;
        let msg = b"250 OK\r\n";
        self.stream.try_write(msg)?;
        Ok(())
    }
}
