use std::str::Utf8Error;

pub mod info;
pub mod players;
pub mod request;

pub fn parse_to_string(data: &[u8], mut index: usize) -> Result<(Box<str>, usize), Utf8Error> {
    let start = index;

    while data[index] != 0 {
        index += 1;
    }

    let string: Box<str> = match std::str::from_utf8(&data[start..index]) {
        Ok(v) => Box::from(v),
        Err(e) => return Err(e),
    };

    index += 1;

    Ok((string, index))
}
