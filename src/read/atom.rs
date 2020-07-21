use std::{u16, str};
use std::error::Error;
use std::io::prelude::*;
use tokio::io::{AsyncReadExt};

/*
 * By "atom", I mean an individual part of a minecraft packet, such as an int, varint, or string.
 */

/*
 * The other methods here are non-async since we'll read a full packet at a time
 * before parsing it, but we need to async read the number at the start of each packet that tells
 * us how long it is before we can read it into a byte buffer
 */
pub async fn read_varint_async<S : AsyncReadExt + Unpin>(source: &mut S) -> Result<i32, Box<dyn Error>> {
    let mut num_read: u64 = 0;
    let mut result: i32 = 0;
    let mut buf = [0; 1]; // 1 byte at a time
    loop {
        let _bytes_read = source.read_exact(&mut buf).await?;
        let byte = buf[0];
        let value = (byte & 0b01111111) as i32;
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


pub fn read_varint(source: &mut impl Read) -> Result<i32, Box<dyn Error>> {
    let mut num_read: u64 = 0;
    let mut result: i32 = 0;
    let mut buf = [0; 1]; // 1 byte at a time
    loop {
        let _bytes_read = source.read_exact(&mut buf)?;
        let byte = buf[0];
        let value = (byte & 0b01111111) as i32;
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

#[cfg(test)]
struct VarIntTestCase(i32, Vec<u8>);

#[cfg(test)]
fn cases() -> Vec<VarIntTestCase> {
    vec![
        VarIntTestCase(0, vec![0x00]),
        VarIntTestCase(1, vec![0x01]),
        VarIntTestCase(255, vec![0xff, 0x01]),
        VarIntTestCase(2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07])
    ]
}

#[tokio::test]
async fn test_read_varint_async() -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;
    for case in cases() {
        let mut buf = Cursor::new(case.1);
        assert_eq!(case.0, read_varint_async(&mut buf).await?);
    }
    Ok(())
}
#[test]
fn test_read_varint() -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;
    for case in cases() {
        let mut buf = Cursor::new(case.1);
        assert_eq!(case.0, read_varint(&mut buf)?);
    }
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

pub fn read_u16(source: &mut impl Read) -> Result<u16, Box<dyn Error>> {
    let mut buf = [0; 2];
    source.read(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

#[test]
fn test_read_u16() -> Result<(), Box<dyn Error>> {
    let mut buf: &[u8] = &(1 as u16).to_be_bytes();
    assert_eq!(1, read_u16(&mut buf)?);
    Ok(())
}

pub fn read_i64(source: &mut impl Read) -> Result<i64, Box<dyn Error>> {
    let mut buf = [0; 8];
    source.read(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

#[test]
fn test_read_i64() -> Result<(), Box<dyn Error>> {
    let mut buf: &[u8] = &(-1 as i64).to_be_bytes();
    assert_eq!(-1, read_i64(&mut buf)?);
    Ok(())
}
