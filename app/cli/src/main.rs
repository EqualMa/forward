#[macro_use]
extern crate clap;

extern crate forward;
extern crate tokio;

use forward::auth::ToAuthentication;
use forward::server::{ForwardServer, ForwardServerConfig};
use forward::target_addr::ToTargetAddr;
use std::io;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "0.1.0", author = "Equal Ma")]
struct Opts {
    /// Use a custom config file if specified, if only '-c' is set, then 'forward.conf' will be used
    // #[clap(short = "c", long = "config", default_value = "forward.conf")]
    // config: Option<String>,

    /// The local address with port to bind
    #[clap(short = "b", long = "bind")]
    bind: String,

    /// forward through proxy if specified
    #[clap(short = "p", long = "proxy")]
    proxy: String,

    /// forward through proxy if specified
    #[clap(short = "t", long = "target")]
    target: String,

    /// proxy auth username if required
    #[clap(short = "U", long = "user")]
    proxy_username: Option<String>,

    /// proxy auth password if required
    #[clap(short = "P", long = "password")]
    proxy_password: Option<String>,
}

async fn run() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    let bind = opts.bind;
    let proxy = opts.proxy;
    let target = opts.target;

    let username = opts.proxy_username;
    let password = opts.proxy_password;

    println!(
        "Proxy {}>>>{}>>>{}\nAuth = {:?} : {:?}",
        bind, proxy, target, username, password
    );

    let proxy_auth = (username, password).to_authentication().unwrap();
    let mut server = ForwardServer::new(ForwardServerConfig {
        bind_addr: bind.parse().unwrap(),
        proxy: proxy.as_str().to_target_addr().unwrap(),
        proxy_auth: proxy_auth,
        target: target.as_str().to_target_addr().unwrap(),
    });

    server.start(None::<tokio::task::JoinHandle<()>>).await
    // server.start().await
}

#[tokio::main]
async fn main() -> io::Result<()> {
    run().await
}
