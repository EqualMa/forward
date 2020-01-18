use super::{TargetAddr, ToTargetAddr};
use std::io;
use std::net::{SocketAddrV4, SocketAddrV6};

impl<'a> ToTargetAddr for &'a str {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        // try to parse as an IP first
        if let Ok(addr) = self.parse::<SocketAddrV4>() {
            return addr.to_target_addr();
        }

        if let Ok(addr) = self.parse::<SocketAddrV6>() {
            return addr.to_target_addr();
        }

        // split the string by ':' and convert the second part to u16
        let mut parts_iter = self.rsplitn(2, ':');
        let port_str = match parts_iter.next() {
            Some(s) => s,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid socket address",
                ))
            }
        };

        let host = match parts_iter.next() {
            Some(s) => s,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid socket address",
                ))
            }
        };

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid port value",
                ))
            }
        };

        (host, port).to_target_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::ToTargetAddr;

    #[test]
    fn str_to_target_addr() {
        assert_eq!(
            "127.0.0.1:8080".to_target_addr().unwrap(),
            ("127.0.0.1", 8080).to_target_addr().unwrap()
        )
        // TODO: v6 & domain test
    }
}
