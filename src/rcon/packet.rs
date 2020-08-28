use std::convert::{TryFrom, TryInto};
use std::i32;
use std::mem;
use std::str;

use tokio::io::AsyncReadExt;

use crate::error::Error;

#[derive(PartialEq, Eq, Debug)]
pub enum PacketType {
    Login = 3,
    Command = 2,
    MultiPacketResponse = 0,
}

impl TryFrom<i32> for PacketType {
    type Error = Error;
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        match val {
            x if x == Self::Login as i32 => Ok(Self::Login),
            x if x == Self::Command as i32 => Ok(Self::Command),
            x if x == Self::MultiPacketResponse as i32 => Ok(Self::MultiPacketResponse),
            _ => Err("Unknown value".into()),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Packet {
    id: i32,
    packet_type: PacketType,
    payload: String, // ASCII characters
}

async fn read_int<S: AsyncReadExt + Unpin>(source: &mut S) -> Result<i32, Error> {
    let mut bytes: [u8; 4] = [0; 4];
    source.read_exact(&mut bytes).await?;
    Ok(i32::from_le_bytes(bytes))
}

const I32_SIZE: usize = mem::size_of::<i32>(); // Always 4 but nice to specify it

pub async fn read<S: AsyncReadExt + Unpin>(source: &mut S) -> Result<Packet, Error> {
    let length: i32 = read_int(source).await?;
    let mut raw_packet = vec![0; length as usize];

    source.read_exact(&mut raw_packet).await?;
    let packet_bytes = &raw_packet[..];

    let (req_id_bytes, packet_bytes) = packet_bytes.split_at(I32_SIZE);
    let request_id = i32::from_le_bytes(req_id_bytes.try_into()?);

    let (packet_type_bytes, packet_bytes) = packet_bytes.split_at(I32_SIZE);
    let packet_type = i32::from_le_bytes(packet_type_bytes.try_into()?).try_into()?;

    let (payload_bytes, null_padding) = packet_bytes.split_at(packet_bytes.len() - 2);

    let payload = str::from_utf8(payload_bytes)?.to_owned();

    assert!(
        null_padding == [0, 0],
        "Packet did not include expected 2 null bytes"
    );

    Ok(Packet {
        id: request_id,
        packet_type,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_read_packet() -> Result<(), Error> {
        let mut raw_packet: Vec<u8> = vec![];
        raw_packet.extend(i32::to_le_bytes(4 + 4 + 2 + "test command".len() as i32).iter());
        raw_packet.extend(i32::to_le_bytes(123).iter()); // request id
        raw_packet.extend(i32::to_le_bytes(2).iter()); // type (2 = Command)
        raw_packet.extend("test command".to_owned().into_bytes().iter());
        raw_packet.extend([0; 2].iter());
        let packet = read(&mut Cursor::new(raw_packet)).await?;
        assert_eq!(
            packet,
            Packet {
                id: 123,
                packet_type: PacketType::Command,
                payload: "test command".to_owned()
            }
        );
        Ok(())
    }
}
