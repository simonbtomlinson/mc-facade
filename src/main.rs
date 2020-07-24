
use tokio::net::{TcpListener, TcpStream};
use std::env;
use crate::error::Error;
use crate::fake_server::run_fake_server;

#[macro_use]
extern crate log;
mod read;
mod write;
mod error;
mod fake_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Starting");
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:25565".to_string());

    run_fake_server(&addr).await?;
    Ok(())
}
