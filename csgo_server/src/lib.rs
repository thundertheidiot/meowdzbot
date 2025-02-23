#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
use std::io::Bytes;
use std::str::Utf8Error;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub mod info;
pub mod players;
pub mod request;

pub fn byte(data: &mut Bytes<&[u8]>) -> Result<u8, Error> {
    Ok(data.next().transpose()?.ok_or("Unexpected end of file")?)
}

pub fn short(data: &mut Bytes<&[u8]>) -> Result<i16, Error> {
    Ok(i16::from_le_bytes(
        data.take(2)
            .collect::<Result<Vec<u8>, std::io::Error>>()?
            .as_slice()
            .try_into()?,
    ))
}

pub fn long(data: &mut Bytes<&[u8]>) -> Result<i32, Error> {
    Ok(i32::from_le_bytes(
        data.take(4)
            .collect::<Result<Vec<u8>, std::io::Error>>()?
            .as_slice()
            .try_into()?,
    ))
}

pub fn float(data: &mut Bytes<&[u8]>) -> Result<f32, Error> {
    Ok(f32::from_le_bytes(
        data.take(4)
            .collect::<Result<Vec<u8>, std::io::Error>>()?
            .as_slice()
            .try_into()?,
    ))
}

pub fn long_long(data: &mut Bytes<&[u8]>) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(
        data.take(8)
            .collect::<Result<Vec<u8>, std::io::Error>>()?
            .as_slice()
            .try_into()?,
    ))
}

pub fn string(data: &mut Bytes<&[u8]>) -> Result<Box<str>, Error> {
    let content: Vec<u8> = data
        .take_while(|b| b.as_ref().map(|b| *b != 0).unwrap_or(false))
        .filter_map(Result::ok)
        .collect();

    Ok(Box::from(std::str::from_utf8(&content)?))
}
