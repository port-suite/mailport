use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::tcp::OwnedWriteHalf,
};

pub struct Quit<'a> {
    stream: &'a mut BufWriter<OwnedWriteHalf>,
}

impl Quit<'_> {
    pub fn new(stream: &mut BufWriter<OwnedWriteHalf>) -> Quit<'_> {
        Quit { stream }
    }

    pub async fn execute(&mut self) -> Result<(), anyhow::Error> {
        let msg = b"221 Bye\r\n";
        self.stream.write_all(msg).await?;
        println!("Got QUIT command; Sending 221 Bye");
        Ok(())
    }
}
