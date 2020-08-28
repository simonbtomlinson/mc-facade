use crate::error::Error;
use crate::server::fake_server::run_fake_server;
use std::env;

#[macro_use]
extern crate log;

mod error;
mod rcon;
mod server;
mod util;
#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Starting");
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25565".to_string());

    run_fake_server(&addr).await?;
    Ok(())
}
