use std::{i64};
use crate::error::Error;
use std::io::Write;
use std::convert::TryInto;

pub fn write_varint(value: i32, sink: &mut impl Write) -> Result<(), Error> {
    // reinterpret the bytes of the value as unsigned so the sign bit shifts along without
    // extension if negative
    let mut value = value as u32;
    let mut iterations = 0;
    loop {
        let mut temp: u8 = (value & 0b01111111) as u8;
        value = value >> 7;
        if value != 0 {
            temp |= 0b10000000;
        }
        sink.write_all(&[temp])?;
        if value == 0 {
            break;
        }
        if iterations > 6 {
            return Err("Too many iterations".into());
        }
        iterations += 1;
    }
    Ok(())
}

#[test]
fn test_write_varint() -> Result<(), Error> {
    use std::io::{SeekFrom, Seek, Cursor};
    for i in [i32::MIN, -1, 0, 1, 2, 1000, 100_000, i32::MAX].iter() {
        let mut cursor = Cursor::new(vec![0; 5]); // varints are at most 5 bytes
        write_varint(*i, &mut cursor)?;
        cursor.seek(SeekFrom::Start(0))?;
        assert_eq!(*i, crate::server::read::atom::read_varint(&mut cursor)?);
    }
    Ok(())
}

pub fn write_i64(value: i64, sink: &mut impl Write) -> Result<(), Error> {
    Ok(sink.write_all(&value.to_be_bytes())?)
}

pub fn write_string(value: &str, sink: &mut impl Write) -> Result<(), Error> {
    write_varint(value.len().try_into()?, sink)?;
    sink.write_all(value.as_bytes())?;
    Ok(())
}
