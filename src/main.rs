
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::env;
use std::error::Error;

use serde::Serialize;

async fn read_varint<S: AsyncReadExt + Unpin>(source: &mut S) -> Result<i64, Box<dyn Error>> {
    let mut num_read: u64 = 0;
    let mut result: i64 = 0;
    let mut buf = [0; 1]; // 1 byte at a time
    loop {
        let bytes_read = source.read(&mut buf).await?;
        let byte = buf[0];
        let value = (byte & 0b01111111) as i64;
        result |= value << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err("VarInt is too big".to_string().into());
        }
        if byte & 0b10000000 == 0 {
            break;
        }
    }
    Ok(result)
}

#[tokio::test]
async fn test_read_varint() -> Result<(), Box<dyn Error>> {
    let buf: &[u8] = &[0xff, 0x01];
    assert_eq!(255, read_varint(&mut buf).await?);

    let buf: &[u8] = &[0xff, 0xff, 0xff, 0xff, 0x07];
    assert_eq!(2147483647, read_varint(&mut buf).await?);
    Ok(())
}

async fn read_string<S: AsyncReadExt + Unpin>(source: &mut S) -> Result<String, Box<dyn Error>> {
    let size = read_varint(source).await? as usize;
    let mut buf: Vec<u8> = vec![0; size];
    source.read_exact(&mut buf).await?;
    unimplemented!();
}

#[derive(Serialize)]
struct Version {
    name: String,
    protocol: u64
}
#[derive(Serialize)]
struct Chat {
    text: String
}
#[derive(Serialize)]
struct Players {
    max: u64,
    online: u64,
    sample: Vec<u64>, // Actually complex but allowed to always be empty
    description: Chat
}


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
