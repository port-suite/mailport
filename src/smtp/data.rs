use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

use crate::smtp::Session;

pub struct Data<'a> {
    reader: &'a mut BufReader<OwnedReadHalf>,
    writer: &'a mut BufWriter<OwnedWriteHalf>,
}

impl Data<'_> {
    pub fn new<'a>(
        reader: &'a mut BufReader<OwnedReadHalf>,
        writer: &'a mut BufWriter<OwnedWriteHalf>,
    ) -> Data<'a> {
        Data { reader, writer }
    }

    pub async fn execute(&mut self, session: &mut Session) -> Result<(), anyhow::Error> {
        self.writer
            .write_all(b"354 End data with <CR>.<CR>\r\n")
            .await?;
        self.writer.flush().await?;
        let mut buf: Vec<u8> = vec![];
        loop {
            let _ = self.reader.read_until(0x0a, &mut buf).await?;
            if buf.ends_with(&[0x0d, 0x0a, 0x2e, 0x0d, 0x0a]) {
                break;
            }
        }
        println!("{buf:?}");
        let message_str = String::from_utf8(buf)?;
        println!("{}", &message_str);
        session.data = Some(message_str);
        self.writer
            .write_all(b"250 OK: NOT IN QUEUE! not yet implemented\r\n")
            .await?;
        self.writer.flush().await?;
        // FIXME This logic does not work
        Ok(())
    }
}
