use std::{u16, str};
use std::error::Error;
use std::io::prelude::*;
use tokio::io::{AsyncReadExt};
use byteorder::{BigEndian, ReadBytesExt};

pub fn read_varint(source: &mut impl Read) -> Result<i64, Box<dyn Error>> {
    let mut num_read: u64 = 0;
    let mut result: i64 = 0;
    let mut buf = [0; 1]; // 1 byte at a time
    loop {
        let _bytes_read = source.read_exact(&mut buf)?;
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

#[test]
fn test_read_varint() -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;
    let mut buf = Cursor::new(vec![0xff, 0x01]);
    assert_eq!(255, read_varint(&mut buf)?);

    let mut buf: &[u8] = &[0xff, 0xff, 0xff, 0xff, 0x07];
    assert_eq!(2147483647, read_varint(&mut buf)?);
    Ok(())
}



pub fn read_string(source: &mut impl Read) -> Result<String, Box<dyn Error>> {
    let size = read_varint(source)? as usize;
    let mut buf: Vec<u8> = vec![0; size];
    source.read_exact(&mut buf)?;
    Ok(str::from_utf8(&buf)?.to_owned())
}

#[test]
fn test_read_string() -> Result<(), Box<dyn Error>> {
    let mut buf: &[u8] = &[0x02, 0x48, 0x49]; // Varint<2>, Utf8<H>, Utf8<I>
    assert_eq!("HI", read_string(&mut buf)?);
    Ok(())
}

pub fn read_short(source: &mut impl Read) -> Result<u16, Box<dyn Error>> {
    let mut buf = [0; 2];
    source.read(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

#[test]
fn test_read_short() -> Result<(), Box<dyn Error>> {
    let mut buf: &[u8] = &(1 as u16).to_be_bytes();
    assert_eq!(1, read_short(&mut buf)?);
    Ok(())
}

//pub async fn read_handshake<S: AsyncReadExt + Unpin>(source: &mut S) -> Result<(), Box<dyn Error>> {
    //let length = read_varint(source).await? as usize;
    //let mut buf: Vec<u8> = vec![0; length];
    //source.read_exact(&mut buf).await?;
    //let mut cursor = std::io::Cursor::new(buf);
    //let packet_id = read_varint(&mut cursor).await?;
    //let server_address = read_string(&mut cursor).await?;
    //let server_port = read_short(&mut cursor).await?;
    //let next_state = read_varint(&mut cursor).await?;
    //Ok(())
//}
//
//pub async fn read_empty_request<S: AsyncReadExt + Unpin>(source: &mut S) -> Result<(), Box<dyn Error>> {
    //let length = read_varint(source).await? as usize;
    //let mut buf: Vec<u8> = vec![0; length];
    //source.read_exact(&mut buf).await?;
    //let mut cursor = std::io::Cursor::new(buf);
    //let packet_id = read_varint(&mut cursor).await?;
    //Ok(())
//}
