mod addr;
mod auth;
mod internal;
pub use internal::forward_tcp_to_socks5;

use self::addr::{read_response, write_addr};
use super::auth::Authentication;
use super::target_addr::{TargetAddr, ToTargetAddr};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;

pub struct Socks5Stream {
    socket: TcpStream,
    proxy_addr: TargetAddr,
}

impl Socks5Stream {
    /// Connects to a target server through a SOCKS5 proxy.
    pub async fn connect<T, U>(
        proxy: T,
        target: U,
        auth: &Authentication,
    ) -> io::Result<Socks5Stream>
    where
        T: ToSocketAddrs,
        U: ToTargetAddr,
    {
        Self::connect_raw(1, proxy, target, auth).await
    }

    /// Connects to a target server through a SOCKS5 proxy using given
    /// username and password.
    pub async fn connect_with_password<T, U>(
        proxy: T,
        target: U,
        username: &str,
        password: &str,
    ) -> io::Result<Socks5Stream>
    where
        T: ToSocketAddrs,
        U: ToTargetAddr,
    {
        let auth = Authentication::Password {
            username: username.to_string(),
            password: password.to_string(),
        };
        Self::connect_raw(1, proxy, target, &auth).await
    }

    pub async fn connect_raw<T, U>(
        command: u8,
        proxy: T,
        target: U,
        auth: &Authentication,
    ) -> io::Result<Socks5Stream>
    where
        T: ToSocketAddrs,
        U: ToTargetAddr,
    {
        let mut socket = TcpStream::connect(proxy).await?;

        let target = target.to_target_addr()?;

        let packet_len = if auth.is_no_auth() { 3 } else { 4 };

        let packet = [
            5,                                     // protocol version
            if auth.is_no_auth() { 1 } else { 2 }, // method count
            auth.id(),                             // method
            0,                                     // no auth (always offered)
        ];

        socket.write_all(&packet[..packet_len]).await?;

        let mut buf = [0; 2];
        socket.read_exact(&mut buf).await?;
        let response_version = buf[0];
        let selected_method = buf[1];

        if response_version != 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid response version",
            ));
        }

        if selected_method == 0xff {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "no acceptable auth methods",
            ));
        }

        if selected_method != auth.id() && selected_method != Authentication::None.id() {
            return Err(io::Error::new(io::ErrorKind::Other, "unknown auth method"));
        }

        match auth {
            Authentication::Password { username, password } if selected_method == auth.id() => {
                auth::password_authentication(&mut socket, username, password).await?
            }
            _ => (),
        }

        write_addr(&mut socket, command, &target).await?;

        let proxy_addr = read_response(&mut socket).await?;

        Ok(Socks5Stream {
            socket: socket,
            proxy_addr: proxy_addr,
        })
    }

    /// Returns the proxy-side address of the connection between the proxy and
    /// target server.
    pub fn proxy_addr(&self) -> &TargetAddr {
        &self.proxy_addr
    }

    /// Returns a shared reference to the inner `TcpStream`.
    pub fn get_ref(&self) -> &TcpStream {
        &self.socket
    }

    /// Returns a mutable reference to the inner `TcpStream`.
    pub fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    /// Consumes the `Socks5Stream`, returning the inner `TcpStream`.
    pub fn into_inner(self) -> TcpStream {
        self.socket
    }
}
