use tokio::net::TcpListener;

use crate::error::Error;
use crate::server::fake_server::run_fake_server;
use std::env;

#[macro_use]
extern crate log;

mod error;
mod rcon;
mod server;
mod proxy;
mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    Ok(())
}
