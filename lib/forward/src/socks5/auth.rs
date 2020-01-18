use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn password_authentication<S: AsyncRead + AsyncWrite + Unpin>(
    socket: &mut S,
    username: &str,
    password: &str,
) -> io::Result<()> {
    if username.len() < 1 || username.len() > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid username",
        ));
    };
    if password.len() < 1 || password.len() > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid password",
        ));
    }

    let mut packet = [0; 515];
    let packet_size = 3 + username.len() + password.len();
    packet[0] = 1; // version
    packet[1] = username.len() as u8;
    packet[2..2 + username.len()].copy_from_slice(username.as_bytes());
    packet[2 + username.len()] = password.len() as u8;
    packet[3 + username.len()..packet_size].copy_from_slice(password.as_bytes());
    socket.write_all(&packet[..packet_size]).await?;

    let mut buf = [0; 2];
    socket.read_exact(&mut buf).await?;
    if buf[0] != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid response version",
        ));
    }
    if buf[1] != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "password authentication failed",
        ));
    }

    Ok(())
}
