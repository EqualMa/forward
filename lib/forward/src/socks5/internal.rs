use super::Socks5Stream;
use crate::auth::Authentication;
use crate::server::pipe;
use crate::target_addr::ToTargetAddr;
use futures::try_join;
use std::io;
use std::net::Shutdown;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;

pub async fn forward_tcp_to_socks5(
    mut client: TcpStream,
    proxy: impl ToSocketAddrs,
    proxy_auth: &Authentication,
    target: impl ToTargetAddr,
) -> io::Result<()> {
    println!("Accepted connection from {:?}", client.peer_addr().unwrap());

    let (client_read, client_write) = client.split();

    // let mut proxy_stream = Socks5Stream::connect(proxy, target).await.unwrap();
    let mut proxy_stream = Socks5Stream::connect(proxy, target, proxy_auth)
        .await?
        .into_inner();

    let (proxy_read, proxy_write) = proxy_stream.split();

    try_join!(
        pipe(client_read, proxy_write),
        pipe(proxy_read, client_write),
    )?;

    println!("[client] joined task done");

    // TODO: better error
    match client.shutdown(Shutdown::Both) {
        Ok(e) => e,
        Err(error) => eprintln!("[client] client <=> here shutdown with failue: {}", error),
    }
    match proxy_stream.shutdown(Shutdown::Both) {
        Ok(e) => e,
        Err(error) => eprintln!("[client] client <=> here shutdown with failue: {}", error),
    }

    Ok(())
}
