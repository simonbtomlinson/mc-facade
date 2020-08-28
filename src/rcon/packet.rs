use std::convert::{TryFrom, TryInto};
use std::i32;
use std::mem;
use std::str;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PacketType {
    Login = 3,
    Command = 2,
    MultiPacketResponse = 0,
    Invalid = 123, // Used on purpose as a follow-up to other packets to delimit multi-packet responses
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
    pub request_id: i32,
    pub packet_type: PacketType,
    pub payload: String, // ASCII characters
}

impl Packet {
    pub fn new(request_id: i32, packet_type: PacketType, payload: String) -> Result<Self, Error> {
        if payload.chars().any(|c| !c.is_ascii()) {
            return Err("payload contains a non-ascii character".into());
        }
        Ok(Packet {
            request_id,
            packet_type,
            payload,
        })
    }
    fn parse(packet_bytes: &[u8]) -> Result<Self, Error> {
        let (req_id_bytes, packet_bytes) = packet_bytes.split_at(I32_SIZE);
        let request_id = i32::from_le_bytes(req_id_bytes.try_into()?);

        let (packet_type_bytes, packet_bytes) = packet_bytes.split_at(I32_SIZE);
        let packet_type = i32::from_le_bytes(packet_type_bytes.try_into()?).try_into()?;

        let (payload_bytes, null_padding) = packet_bytes.split_at(packet_bytes.len() - 2);

        // TODO: this actually is supposed to be ascii only, luckily ascii and utf8 line up for common text
        let payload = str::from_utf8(payload_bytes)?.to_owned();

        debug_assert!(
            null_padding == [0, 0],
            "Packet did not include expected 2 null bytes"
        );

        Ok(Self {
            request_id,
            packet_type,
            payload,
        })
    }
    fn serialize(&self) -> Vec<u8> {
        // 2 i32s, the string, 2 bytes null padding
        let length = I32_SIZE * 2 + self.payload.len() + 2;
        // payload length + length element, we know the length so why not preallocate
        let mut dest = Vec::with_capacity(length + I32_SIZE);
        dest.extend_from_slice(&i32::to_le_bytes(length as i32));
        dest.extend_from_slice(&i32::to_le_bytes(self.request_id));
        dest.extend_from_slice(&i32::to_le_bytes(self.packet_type.clone() as i32));
        dest.extend(self.payload.bytes());
        dest.extend_from_slice(&[0, 0]); // null padding
        let dest_len = dest.len();
        debug_assert!(dest_len == length + I32_SIZE);
        dest
    }
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

    Packet::parse(&packet_bytes)
}

pub async fn write<W: AsyncWriteExt + Unpin>(packet: &Packet, dest: &mut W) -> Result<(), Error> {
    let data = packet.serialize();
    Ok(dest.write_all(&data).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    type TestResult = Result<(), Error>;

    #[test]
    fn test_parse_packet() -> TestResult {
        let mut raw_packet: Vec<u8> = vec![];
        raw_packet.extend_from_slice(&i32::to_le_bytes(123)); // request id
        raw_packet.extend_from_slice(&i32::to_le_bytes(2)); // type (2 = Command)
        raw_packet.extend("test command".bytes()); // payload
        raw_packet.extend_from_slice(&[0; 2]); // padding
        let packet = Packet::parse(&raw_packet)?;
        assert_eq!(
            packet,
            Packet {
                request_id: 123,
                packet_type: PacketType::Command,
                payload: "test command".to_owned()
            }
        );
        Ok(())
    }

    #[test]
    fn test_serialize_packet() -> TestResult {
        let packet = Packet {
            request_id: -5,
            packet_type: PacketType::Login,
            payload: "this would be a password".into(),
        };
        let packet_bytes_with_len = packet.serialize();
        // The parse doesn't expect a length
        let deserialized_packet = Packet::parse(&packet_bytes_with_len[I32_SIZE..])?;
        assert_eq!(packet, deserialized_packet);
        Ok(())
    }
}
