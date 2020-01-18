pub mod auth;
pub mod server;
pub mod socks5;
pub mod target_addr;

pub use futures::executor::block_on;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
