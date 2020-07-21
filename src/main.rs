
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::env;
use std::error::Error;

use serde::Serialize;

mod read;
mod write;
async fn handle_connection(mut socket: TcpStream) {
    let mut buf = [0; 1024];
    socket.read(&mut buf).await.expect("failed to read from socket");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:25565".to_string());

    let mut listener = TcpListener::bind(&addr).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }

    Ok(())
}
