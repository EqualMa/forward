pub mod auth;
pub mod server;
pub mod socks5;
pub mod target_addr;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
