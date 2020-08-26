
use super::atom;
use crate::error::Error;

use std::io::{Read, Cursor};

use tokio::io::AsyncReadExt;

#[derive(Debug, Eq, PartialEq)]
pub enum Packet {
    Handshake(Handshake),
    HandshakeRequest(HandshakeRequest),
    Ping(Ping)
}

pub async fn read<S : AsyncReadExt + Unpin>(source: &mut S) -> Result<Packet, Error> {
    let length = atom::read_varint_async(source).await? as usize;
    let mut buf = vec![0; length];
    if length > 0 {
        source.read_exact(&mut buf).await?;
    }
    let mut cursor = Cursor::new(buf);

    let packet_id = atom::read_varint(&mut cursor)?;
    trace!("reading packet type {:#}", packet_id);
    match packet_id {
        Handshake::ID => match length {
            // Empty packet with 1-byte packet id has length 1 (for the packet id)
            1 => Ok(Packet::HandshakeRequest(HandshakeRequest {})),
            _ => Ok(Packet::Handshake(Handshake::decode(&mut cursor)?))
        },
        Ping::ID => Ok(Packet::Ping(Ping::decode(&mut cursor)?)),
        id => Err(format!("Unknown packet id {}", id).into())
    }
}

#[cfg(test)]
type AsyncTestResult = Result<(), Error>;

#[tokio::test]
async fn test_read_handshake() -> AsyncTestResult {
    let mut buf: Vec<u8> = vec![0x10,0x00,0xe0,0x05,0x09,0x6c,0x6f,0x63,0x61,0x6c,0x68,0x6f,0x73,0x74,0x1f,0x90,0x01];
    let mut cursor = Cursor::new(&mut buf);
    let expected = Handshake {
        protocol_version: 736,
        server_address: "localhost".to_owned(),
        server_port: 8080,
        next_state: 1
    };
    assert_eq!(Packet::Handshake(expected), read(&mut cursor).await?);
    Ok(())
}

#[tokio::test]
async fn test_read_handshake_request() -> AsyncTestResult {
    let mut buf: Vec<u8> = vec![0x01, 0x00];
    let mut cursor = Cursor::new(&mut buf);
    assert_eq!(Packet::HandshakeRequest(HandshakeRequest {}), read(&mut cursor).await?);
    Ok(())
}


#[tokio::test]
async fn test_read_ping() -> AsyncTestResult {
    let mut buf: Vec<u8> = vec![0x09, 0x01];
    buf.extend_from_slice(&((123 as i64).to_be_bytes()));
    let mut cursor = Cursor::new(&mut buf);
    assert_eq!(Packet::Ping(Ping { payload: 123 }), read(&mut cursor).await?);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32
}

impl Handshake {
    const ID: i32 = 0x00;
    fn decode(source: &mut impl Read) -> Result<Self, Error> {
        Ok(Handshake {
            protocol_version: atom::read_varint(source)?,
            server_address: atom::read_string(source)?,
            server_port: atom::read_u16(source)?,
            next_state: atom::read_varint(source)?
        })
    }
}

// Minecraft sends a 0x00 empty request after the hanshake packet
#[derive(Debug, Eq, PartialEq)]
pub struct HandshakeRequest {}

#[derive(Debug, Eq, PartialEq)]
pub struct Ping {
    pub payload: i64
}

impl Ping {
    const ID: i32 = 0x01;

    fn decode(source: &mut impl Read) -> Result<Self, Error> {
        Ok(Ping {
            payload: atom::read_i64(source)?
        })
    }
}
