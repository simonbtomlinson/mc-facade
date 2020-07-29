
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, delay_for};
use crate::error::Error;

use crate::read::packet::*;

use crate::write::packet::{write, Pong, HandshakeResponse, LoginDisconnect};
use crate::util::race::{race, RaceResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

enum ConnectionResult {
    Login,
    ServerListPing
}

async fn handle_connection(mut socket: TcpStream) -> Result<ConnectionResult, Error> {
    debug!("Starting to handle a connection");
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


/// Run a fake server until someone logs in
pub async fn run_fake_server(addr: &str) -> Result<(), Error> {
    let should_start_server = Arc::new(AtomicBool::new(false));
    let mut listener = TcpListener::bind(&addr).await?;
    while should_start_server.load(Ordering::Relaxed) == false {
        match race(listener.accept(), delay_for(Duration::from_secs(3))).await {
            RaceResult::Left(listener_result) => {
                debug!("Got a socket connection");
                let (socket, _) = listener_result?;
                let should_start_server = should_start_server.clone();
                tokio::spawn(async move {
                    match handle_connection(socket).await {
                        Ok(ConnectionResult::Login) => {
                            info!("Finished a login");
                            should_start_server.store(true, Ordering::Relaxed);
                        },
                        Ok(ConnectionResult::ServerListPing) => info!("Finished a server list ping"),
                        Err(e) => error!("{}", e)
                    }
                });
            },
            RaceResult::Right(_) => info!("Timeout"),
        }
    }
    Ok(())
}
