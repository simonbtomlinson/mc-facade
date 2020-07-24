use super::atom;
use std::io::Write;
use tokio::io::AsyncWriteExt;
use std::convert::TryInto;
use crate::error::Error;

pub trait Packet {
    const ID: i32;
    fn write_to(&self, sink: &mut impl Write) -> Result<(), Error>;
}


pub async fn write<P : Packet, W: AsyncWriteExt + Unpin>(packet: &P, dest: &mut W) -> Result<(), Error> {
    let mut buf = vec![];
    atom::write_varint(P::ID, &mut buf)?; // Every packet has an ID so write it for the packet
    packet.write_to(&mut buf)?;
    let mut size_buf = vec![];
    atom::write_varint(buf.len().try_into()?, &mut size_buf)?;
    // It would definitely be better to do these writes together, but this works for now
    dest.write_all(&size_buf).await?;
    dest.write_all(&buf).await?;
    Ok(())
}

#[tokio::test]
async fn test_write_packet() -> Result<(), Error> {
    let packet = Pong { payload: 12345 };
    let mut size_buf = vec![];
    atom::write_varint(HandshakeResponse::ID, &mut size_buf)?;
    let expected_size: i32 = 1 + 8; // 1 for the id, 8 for the (long) payload
    let mut buf = vec![];
    let mut expected_buf = vec![];
    write(&packet, &mut buf).await?;
    atom::write_varint(expected_size, &mut expected_buf)?;
    atom::write_varint(Pong::ID, &mut expected_buf)?;
    packet.write_to(&mut expected_buf)?;
    assert_eq!(expected_buf, buf);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct HandshakeResponse {
    pub version_name: String,
    pub protocol: i32,
    pub max_players: u32,
    pub online_players: u32,
    pub description: String
}

impl Packet for HandshakeResponse {
    const ID: i32 = 0x00;
    fn write_to(&self, sink: &mut impl Write) -> Result<(), Error> {
        // This is the only place I need to make json so I don't really need something as
        // heavyweight as serde for this.
        let json = format!(r#"{{
            "version": {{
                "name": "{version_name}",
                "protocol": {protocol}
            }},
            "players": {{
                "max": {max_players},
                "online": {online_players},
                "sample": []
            }},
            "description": {{
                "text": "{description}"
            }}
        }}"#,
                version_name=self.version_name,
                protocol=self.protocol,
                max_players=self.max_players,
                online_players=self.online_players,
                description=self.description
        );
        atom::write_string(&json, sink)?;
        Ok(())
    }
}


#[derive(Debug, Eq, PartialEq)]
pub struct Pong {
    pub payload: i64
}

impl Packet for Pong {
    const ID: i32 = 0x01;
    fn write_to(&self, sink: &mut impl Write) -> Result<(), Error> {
        atom::write_i64(self.payload, sink)?;
        Ok(())
    }
}
