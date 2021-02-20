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
    proxy::run_proxy("127.0.0.1:9000", "localhost:25565").await
}
