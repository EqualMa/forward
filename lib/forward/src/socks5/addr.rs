use crate::target_addr::TargetAddr;
use byteorder::{BigEndian, WriteBytesExt};
use std::io;
use std::io::Write;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

const MAX_ADDR_LEN: usize = 260;

pub async fn read_addr<R: AsyncRead + Unpin>(socket: &mut R) -> io::Result<TargetAddr> {
    match socket.read_u8().await? {
        1 => {
            let ip = Ipv4Addr::from(socket.read_u32().await?); // implied BigEndian
            let port = socket.read_u16().await?; // implied BigEndian
            Ok(TargetAddr::Ip(SocketAddr::V4(SocketAddrV4::new(ip, port))))
        }
        3 => {
            let len = socket.read_u8().await?;
            let mut domain = vec![0; len as usize];
            socket.read_exact(&mut domain).await?;
            let domain = String::from_utf8(domain)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let port = socket.read_u16().await?; // implied BigEndian
            Ok(TargetAddr::Domain(domain, port))
        }
        4 => {
            let mut ip = [0; 16];
            socket.read_exact(&mut ip).await?;
            let ip = Ipv6Addr::from(ip);
            let port = socket.read_u16().await?; // implied BigEndian
            Ok(TargetAddr::Ip(SocketAddr::V6(SocketAddrV6::new(
                ip, port, 0, 0,
            ))))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "unsupported address type",
        )),
    }
}

pub async fn write_addr<R: AsyncWrite + Unpin>(
    socket: &mut R,
    command: u8,
    target: &TargetAddr,
) -> io::Result<()> {
    let mut packet = [0; MAX_ADDR_LEN + 3];
    packet[0] = 5; // protocol version
    packet[1] = command; // command
    packet[2] = 0; // reserved
    let len = write_addr_to_packet(&mut packet[3..], target)?;
    socket.write_all(&packet[..len + 3]).await?;
    Ok(())
}

fn write_addr_to_packet(mut packet: &mut [u8], target: &TargetAddr) -> io::Result<usize> {
    let start_len = packet.len();
    match target {
        TargetAddr::Ip(SocketAddr::V4(addr)) => {
            packet.write_u8(1).unwrap();
            packet.write_u32::<BigEndian>((*addr.ip()).into()).unwrap();
            packet.write_u16::<BigEndian>(addr.port()).unwrap();
        }
        TargetAddr::Ip(SocketAddr::V6(addr)) => {
            packet.write_u8(4).unwrap();
            packet.write_all(&addr.ip().octets()).unwrap();
            packet.write_u16::<BigEndian>(addr.port()).unwrap();
        }
        TargetAddr::Domain(ref domain, port) => {
            packet.write_u8(3).unwrap();
            if domain.len() > u8::max_value() as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "domain name too long",
                ));
            }
            packet.write_u8(domain.len() as u8).unwrap();
            packet.write_all(domain.as_bytes()).unwrap();
            packet.write_u16::<BigEndian>(*port).unwrap();
        }
    }

    Ok(start_len - packet.len())
}

pub async fn read_response<R: AsyncRead + Unpin>(socket: &mut R) -> io::Result<TargetAddr> {
    let mut socket = BufReader::with_capacity(MAX_ADDR_LEN + 3, socket);
    if socket.read_u8().await? != 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid response version",
        ));
    }
    match socket.read_u8().await? {
        0 => {}
        1 => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "general SOCKS server failure",
            ))
        }
        2 => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "connection not allowed by ruleset",
            ))
        }
        3 => return Err(io::Error::new(io::ErrorKind::Other, "network unreachable")),
        4 => return Err(io::Error::new(io::ErrorKind::Other, "host unreachable")),
        5 => return Err(io::Error::new(io::ErrorKind::Other, "connection refused")),
        6 => return Err(io::Error::new(io::ErrorKind::Other, "TTL expired")),
        7 => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "command not supported",
            ))
        }
        8 => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "address kind not supported",
            ))
        }
        _ => return Err(io::Error::new(io::ErrorKind::Other, "unknown error")),
    }
    if socket.read_u8().await? != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid reserved byte",
        ));
    }

    read_addr(&mut socket).await
}
