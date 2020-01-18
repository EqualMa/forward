#[macro_use]
extern crate clap;

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

    /// proxy auth user if required
    #[clap(short = "u", long = "user")]
    proxy_user: Option<String>,

    /// proxy auth password if required
    #[clap(short = "p", long = "password")]
    proxy_user: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    let bind = opts.bind;
    let proxy = opts.proxy;

    println!("Proxy {} {}", bind, proxy);
}
