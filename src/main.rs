use crate::error::Error;
use crate::server::fake_server::run_fake_server;
use std::env;

#[macro_use]
extern crate log;

mod error;
mod gcloud;
mod rcon;
mod server;
mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}
