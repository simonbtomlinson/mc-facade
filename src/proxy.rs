use core::future;
use std::sync::Arc;
use futures::{future::{AbortHandle, Abortable, join, select}, pin_mut};
use tokio::{io::copy, net::{TcpListener, TcpStream}, sync::watch};
use watch::{Receiver, Sender};

use crate::error::Error;

async fn proxy_to_remote(incoming: TcpStream, outgoing: TcpStream) {
    let (mut inc_reader, mut inc_writer) = incoming.into_split();
    let (mut out_reader, mut out_writer) = outgoing.into_split();
    let write_to_outgoing = copy(&mut inc_reader, &mut out_writer);
    let read_from_incoming = copy(&mut out_reader, &mut inc_writer);
    join(read_from_incoming, write_to_outgoing).await;
}

struct Proxy {
    remote_addr: &'static str,
    cancels: Vec<AbortHandle>
}

impl Proxy {
    fn new(remote_addr: &'static str) -> Self {
        Self { remote_addr, cancels: Vec::new() }
    }

    fn proxy_conn(&mut self, conn: TcpStream) {
        let remote_addr = self.remote_addr.clone();
        let (handle, reg) = AbortHandle::new_pair();
        self.cancels.push(handle);
        let fut = Abortable::new(async move {
            let remote_side = TcpStream::connect(remote_addr).await.unwrap();
            proxy_to_remote(conn, remote_side).await;
        }, reg);
        tokio::spawn(fut);
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.cancels.iter().for_each(AbortHandle::abort);
    }
}

pub async fn run_proxy(addr: &str, remote_addr: &'static str) -> Result<(), Error> {
    let mut listener = TcpListener::bind(addr).await?;
    info!("Running proxy on {}", addr);


    loop {
        let (incoming, _sock) = listener.accept().await?;
        tokio::spawn(async move {
            match TcpStream::connect(&remote_addr).await {
                Ok(outgoing) => proxy_to_remote(incoming, outgoing).await,
                Err(err) => { error!("{}", err)}
            }
        });
    }

    Ok(())
}