use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::tcp::OwnedWriteHalf,
};

pub struct Noop<'a> {
    stream: &'a mut BufWriter<OwnedWriteHalf>,
}

impl Noop<'_> {
    pub fn new(stream: &mut BufWriter<OwnedWriteHalf>) -> Noop<'_> {
        Noop { stream }
    }

    pub async fn execute(&mut self) -> Result<(), anyhow::Error> {
        println!("Got NOOP command; Sending 250 OK");
        let msg = b"250 OK\r\n";
        self.stream.write_all(msg).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
