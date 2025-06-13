use std::io::{Error, ErrorKind};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn write_varint(buffer: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        buffer.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
    buffer.push((value & 0x7F) as u8);
}

pub fn write_string(buffer: &mut Vec<u8>, val: &str) {
    let length = val.len();
    write_varint(buffer, length as u32);

    buffer.extend_from_slice(val.as_bytes());
}

pub fn write_ushort(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_uuid(buffer: &mut Vec<u8>, uuid: &Uuid) {
    let uuid_bytes = uuid.as_bytes();
    buffer.extend_from_slice(uuid_bytes);
}

pub fn read_ushort(buffer: &Vec<u8>, position: &mut usize) -> Result<u16, Error> {
    if *position + 2 > buffer.len() {
        return Err(Error::new(ErrorKind::Other, "Not enough bytes"));
    }

    let result = u16::from_be_bytes([buffer[*position], buffer[*position + 1]]);

    *position += 2;

    Ok(result)
}

pub fn read_varint(buffer: &[u8], position: &mut usize) -> Result<u32, Error> {
    let mut value: u32 = 0;
    let mut shift: u32 = 0;

    loop {
        if *position >= buffer.len() {
            return Err(Error::new(ErrorKind::Other, "Not enough bytes"));
        }

        let current_byte = buffer[*position];
        *position += 1;

        value |= ((current_byte & 0x7F) as u32) << shift;
        if (current_byte & 0x80) == 0 {
            break;
        }

        shift += 7;

        if shift >= 32 {
            return Err(Error::new(ErrorKind::Other, "Varint too big!"))?;
        }
    } // There is probably a better way to read a varint

    Ok(value)
}

pub fn varint_size(mut value: u32) -> usize {
    let mut size = 0;
    loop {
        size += 1;
        if value & !0x7F == 0 {
            break;
        }
        value >>= 7;
    }
    size
}

pub fn read_string(buffer: &[u8], position: &mut usize) -> Result<String, Error> {
    let length = read_varint(buffer, position)? as usize;

    if *position + length > buffer.len() {
        return Err(Error::new(ErrorKind::Other, "Not enoguh bytes"))?;
    }

    let string_bytes = &buffer[*position..*position + length];
    *position += length;
    match String::from_utf8(string_bytes.to_vec()) {
        Ok(string) => Ok(string),
        Err(_) => Err(Error::new(ErrorKind::Other, "Invalid UTF-8 in string"))?,
    }
}

pub fn read_uuid(buffer: &[u8], position: &mut usize) -> Result<Uuid, Error> {
    if *position + 16 > buffer.len() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
    }

    let uuid_bytes: [u8; 16] = buffer[*position..*position + 16].try_into().unwrap();
    *position += 16;

    Ok(Uuid::from_bytes(uuid_bytes))
}

pub fn write_bool(buffer: &mut Vec<u8>, value: bool) {
    buffer.push(if value { 1 } else { 0 });
}

pub fn read_bool(buffer: &Vec<u8>, position: &mut usize) -> Result<bool, Error> {
    if *position >= buffer.len() {
        return Err(Error::new(ErrorKind::Other, "Not enough bytes"));
    }

    let result = buffer[*position] != 0;
    *position += 1;

    Ok(result)
}

pub fn write_long(buffer: &mut Vec<u8>, payload: u64) {
    buffer.extend_from_slice(&payload.to_be_bytes());
}

pub fn read_long(buffer: &[u8], position: &mut usize) -> Result<u64, Error> {
    if buffer.len() < *position + 8 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
    }

    let slice = &buffer[*position..*position + 8];
    let result = u64::from_be_bytes(slice.try_into().unwrap());

    *position += 8;
    Ok(result)
}

use fastnbt::from_bytes;
use fastnbt::to_bytes;

pub fn write_nbt<T: Serialize>(buffer: &mut Vec<u8>, value: &T) -> Result<(), Error> {
    let payload = to_bytes(value)
        .map_err(|e| Error::new(ErrorKind::Other, format!("NBT write error: {e}")))?;
    buffer.extend(payload);
    Ok(())
}

pub fn read_nbt<'de, T: Deserialize<'de>>(
    buffer: &'de [u8],
    position: &mut usize,
) -> Result<T, Error> {
    let slice = &buffer[*position..];
    let result: T = from_bytes(slice)
        .map_err(|e| Error::new(ErrorKind::Other, format!("NBT read error: {e}")))?;

    *position = buffer.len(); // Consumes all bytes, will this be problematic?
    Ok(result)
}

pub fn read_byte(buffer: &[u8], position: &mut usize) -> Result<u8, Error> {
    if *position >= buffer.len() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
    }

    let byte = buffer[*position];
    *position += 1;
    Ok(byte)
}

pub fn write_byte(buffer: &mut Vec<u8>, value: u8) {
    buffer.push(value);
}

pub fn read_bytes(
    buffer: &[u8],
    position: &mut usize,
    data_length: usize,
) -> Result<Vec<u8>, Error> {
    if *position + data_length > buffer.len() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
    }

    let slice = buffer[*position..*position + data_length].to_vec();
    *position += data_length;
    Ok(slice)
}
