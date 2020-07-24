
use tokio::net::{TcpListener, TcpStream};
use std::env;
use crate::error::Error;

#[macro_use]
extern crate log;
mod read;
mod write;
mod error;

use read::packet::*;

use write::packet::{write, Pong, HandshakeResponse, LoginDisconnect};

enum ConnectionResult {
    Login,
    ServerListPing
}

async fn handle_connection(mut socket: TcpStream) -> Result<ConnectionResult, Error> {
    if let Packet::Handshake(handshake) = read(&mut socket).await? { // first a handshake
        debug!("Got a handshake packet");
        if handshake.next_state == 2 { // login request
            debug!("packet is a login packet");
            write(&LoginDisconnect { reason: "Starting the real server, this could take a bit" }, &mut socket).await?;
            return Ok(ConnectionResult::Login);
        }
        debug!("packet is a server list ping packet");
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
                debug!("Got a ping");
                write(&Pong { payload: ping.payload }, &mut socket).await?;
            }
            return Ok(ConnectionResult::ServerListPing)
        }
    }
    Err("Not a handshake packet".into())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Starting");
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:25565".to_string());

    let mut listener = TcpListener::bind(&addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            match handle_connection(socket).await {
                Ok(ConnectionResult::Login) => info!("Finished a login"),
                Ok(ConnectionResult::ServerListPing) => info!("Finished a server list ping"),
                Err(e) => error!("{}", e)
            }
        });
    }
}
