use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn pipe<R, W>(mut read: R, mut write: W) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    println!("[client] client => here => proxy => server: start");
    let mut buf = [0; 4096];
    loop {
        let n = read.read(&mut buf).await?;
        if n <= 0 {
            break;
        }

        write.write_all(&mut buf[..n]).await?;
    }
    println!("[client] client => here => proxy => server: end");
    Ok(())
}
