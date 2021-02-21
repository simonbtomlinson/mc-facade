use core::future;
use std::sync::Arc;
use futures::{future::{AbortHandle, Abortable, join, select}, pin_mut};
use tokio::{io::copy, net::{TcpListener, TcpStream, ToSocketAddrs}, sync::watch};
use watch::{Receiver, Sender};

use crate::error::Error;

async fn proxy_to_remote(incoming: TcpStream, outgoing: TcpStream) {
    let (mut inc_reader, mut inc_writer) = incoming.into_split();
    let (mut out_reader, mut out_writer) = outgoing.into_split();
    let write_to_outgoing = copy(&mut inc_reader, &mut out_writer);
    let read_from_incoming = copy(&mut out_reader, &mut inc_writer);
    join(read_from_incoming, write_to_outgoing).await;
}
pub async fn proxy<A: ToSocketAddrs>(incoming: TcpStream, remote_addr: A) -> Result<(), Error> {
    let outgoing = TcpStream::connect(remote_addr).await?;
    proxy_to_remote(incoming, outgoing).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    // Binding to port 0 makes the os allocate a free high port, so we can run this test without worrying about ports
    async fn mk_listener() -> TcpListener {
        TcpListener::bind("127.0.0.1:0").await.unwrap()
    }

    #[tokio::test]
    async fn test_proxy_proxies() {
        // steps for this test:
        // Start a tcp server that reads a number and returns that number + 1
        // Start a tcp server that proxies to that server
        // Run a connection to the proxy server, send it 1, expect to get 2 back.

        let mut real_listener = mk_listener().await;
        let real_addr = real_listener.local_addr().unwrap();

        // The real server - adds one to the number sent
        tokio::spawn(async move {
            let mut stream = real_listener.accept().await.unwrap().0;
            let num = stream.read_i64().await.unwrap();
            stream.write_all(&(num + 1).to_be_bytes()).await.unwrap();
        });

        let mut proxy_listener = mk_listener().await;
        let proxy_addr = proxy_listener.local_addr().unwrap();
        // The proxy - forwards to the real address
        tokio::spawn(async move {
            let mut stream = proxy_listener.accept().await.unwrap().0;
            proxy(stream, real_addr).await
        });

        // Connect to the proxy and make sure that our number goes through correctly
        let mut stream = TcpStream::connect(proxy_addr).await.unwrap();
        let send_num: i64 = 1;
        stream.write_all(&send_num.to_be_bytes()).await.unwrap();
        let recv_num = stream.read_i64().await.unwrap();
        assert_eq!(send_num + 1, recv_num);
    }
}