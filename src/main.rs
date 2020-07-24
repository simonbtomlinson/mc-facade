
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::env;
use crate::error::Error;
use std::net::Shutdown;

use serde::Serialize;

mod read;
mod write;
mod error;

use read::packet::*;

use write::packet::{write, Pong, HandshakeResponse};

async fn handle_connection(mut socket: TcpStream) -> Result<(), Error> {
    if let Packet::Handshake(handshake) = read(&mut socket).await? { // first a handshake
        if handshake.next_state == 2 { // login request, we can't handle this yet
            socket.shutdown(Shutdown::Both)?;
        }
        // Then a request for a response (no idea why these aren't the same)
        if let Packet::HandshakeRequest(handshake_request) = read(&mut socket).await? {
            write(&HandshakeResponse {
                protocol: handshake.protocol_version,
                version_name: "test".to_owned(), // Need to fake this based on the request
                description: "Fake!".to_owned(),
                max_players: 1,
                online_players: 0
            }, &mut socket).await?;
            if let Packet::Ping(ping) = read(&mut socket).await? {
                write(&Pong { payload: ping.payload }, &mut socket).await?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:25565".to_string());

    let mut listener = TcpListener::bind(&addr).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(socket).await.unwrap();
        });
    }

    Ok(())
}
