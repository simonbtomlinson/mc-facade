use std::{i64};
use std::error::Error;
use std::io::Write;

pub fn write_varint(value: i32, sink: &mut impl Write) -> Result<(), Box<dyn Error>> {
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
        eprintln!("Value: {:#}, temp: {:#}", value, temp);
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
fn test_write_varint() -> Result<(), Box<dyn Error>> {
    use std::io::{SeekFrom, Seek, Cursor};
    for i in [i32::MIN, -1, 0, 1, 2, 1000, 100_000, i32::MAX].iter() {
        let mut cursor = Cursor::new(vec![0; 5]); // varints are at most 5 bytes
        write_varint(*i, &mut cursor)?;
        cursor.seek(SeekFrom::Start(0))?;
        assert_eq!(*i, crate::read::atom::read_varint(&mut cursor)?);
    }
    Ok(())
}

pub fn write_i64(value: i64, sink: &mut impl Write) -> Result<(), Box<dyn Error>> {
    Ok(sink.write_all(&value.to_be_bytes())?)
}
