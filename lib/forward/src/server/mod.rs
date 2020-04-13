use crate::auth::Authentication;
use crate::socks5::forward_tcp_to_socks5;
use crate::target_addr::TargetAddr;
use futures::future::{try_join_all, FutureExt};
use futures::select;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;

mod pipe;
pub use pipe::pipe;

#[derive(Debug, PartialEq)]
pub struct ForwardServerConfig {
    pub bind_addr: SocketAddr,
    pub proxy: TargetAddr,
    pub proxy_auth: Authentication,
    pub target: TargetAddr,
}

#[derive(Debug)]
pub enum ForwardServerState {
    Started,
    Stopping,
    Stopped,
}

#[derive(Debug)]
pub struct ForwardServer {
    tasks: Vec<JoinHandle<io::Result<()>>>,
    // tasks: Vec<dyn std::future::Future<Output = io::Result<()>>>,
    state: ForwardServerState,
    state_tx: watch::Sender<u8>,
    state_rx: watch::Receiver<u8>,
    config: ForwardServerConfig,
}

const STATE_STARTED: u8 = 1;
const STATE_STOPPING: u8 = 2;
const STATE_STOPPED: u8 = 0;

impl ForwardServer {
    pub fn new(config: ForwardServerConfig) -> ForwardServer {
        let (tx, rx) = watch::channel(0);

        ForwardServer {
            tasks: vec![],
            config: config,
            state: ForwardServerState::Stopped,
            state_tx: tx,
            state_rx: rx,
        }
    }

    pub async fn start<T>(&mut self, stopper: Option<T>) -> io::Result<()>
    where
        T: std::future::Future + Send + Unpin + 'static,
        T::Output: Send + 'static,
    {
        match self.state {
            ForwardServerState::Stopped => {
                self.state = ForwardServerState::Started;
                // TODO:
                match self.state_tx.broadcast(STATE_STARTED) {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
            // ForwardServerState::Stopping => {}
            _ => return Ok(()),
        }

        // TODO .expect(&format!("Address {} is already in use", addr));
        let bind_addr = self.config.bind_addr;

        let mut listener = TcpListener::bind(bind_addr).await?;
        // println!("Server running on {}", bind_addr);

        let mut stopper = match stopper {
            Some(s) => Some(s.fuse()),
            None => None,
        };
        loop {
            let accepted = match stopper.as_mut() {
                None => listener.accept().await,
                Some(mut s) => select! {
                    accept = listener.accept().fuse() => accept,
                    recv = s => {
                        self.state = ForwardServerState::Stopping;
                        // TODO:
                        match self.state_tx.broadcast(STATE_STOPPING) {
                            Ok(_) => {}
                            Err(_) => {}
                        };

                        break;
                    },
                },
            };
            // try_select(to_accept, self.state_rx.recv()).await?;

            let (socket, socket_addr) = accepted?;
            println!("Accepted at {}", socket_addr);

            let proxy = self.config.proxy.clone();
            let proxy_auth = self.config.proxy_auth.clone();
            let target = self.config.target.clone();

            self.tasks.push(tokio::spawn(async move {
                match proxy {
                    TargetAddr::Ip(s) => {
                        forward_tcp_to_socks5(socket, s, &proxy_auth, target).await
                    }
                    TargetAddr::Domain(d, p) => {
                        forward_tcp_to_socks5(socket, (d.as_str(), p), &proxy_auth, target).await
                    }
                }
            }));
        }

        // TODO: error handling
        println!("SERVER try_join_all...");
        let res = try_join_all(&mut self.tasks).await;
        println!("SERVER try_join_all done!");

        self.tasks = vec![];

        res?;

        self.state = ForwardServerState::Stopped;
        // TODO:
        match self.state_tx.broadcast(STATE_STOPPED) {
            Ok(_) => {}
            Err(_) => {}
        }

        Ok(())
    }

    pub fn query_state(&self) -> &ForwardServerState {
        &self.state
    }

    pub async fn stopped(&mut self) {
        self.wait_till_state(STATE_STOPPED).await;
    }

    async fn wait_till_state(&mut self, state: u8) {
        while let Some(value) = self.state_rx.recv().await {
            if value == state {
                break;
            }
        }
    }
}
