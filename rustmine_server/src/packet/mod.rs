use std::{
    any::Any,
    io::{Error, ErrorKind, Read, Write},
    sync::Arc,
};

use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod clientbound;
pub mod serverbound;

pub mod data {
    use std::io::{Error, ErrorKind};

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

    pub(crate) fn read_long(buffer: &[u8], position: &mut usize) -> Result<u64, Error> {
        if buffer.len() < *position + 8 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }

        let slice = &buffer[*position..*position + 8];
        let result = u64::from_be_bytes(slice.try_into().unwrap());

        *position += 8;
        Ok(result)
    }
}

pub type RawPacket = (u32, u32, Vec<u8>);
pub async fn read_packet(
    stream: &mut TcpStream,
    compression_threshold: u32,
) -> Result<RawPacket, Box<dyn std::error::Error>> {
    let mut temp_buffer = vec![0u8; 5];
    stream.read_exact(&mut temp_buffer[..1]).await?;

    let mut position = 0;
    let mut slice = &temp_buffer[..];
    let packet_length = data::read_varint(&mut slice, &mut position)?;

    if packet_length == 0 {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidData,
            "Packet length cannot be zero",
        )));
    }

    let mut buffer = vec![0u8; packet_length as usize];
    stream.read_exact(&mut buffer).await?;

    let mut position = 0;
    let mut slice = &buffer[..];

    if compression_threshold == 0 {
        let packet_id = data::read_varint(&mut slice, &mut position)?;
        let data = slice[position..].to_vec();
        return Ok((packet_length, packet_id, data));
    }

    // Read Data Length
    let data_length = data::read_varint(&mut slice, &mut position)?;

    let (packet_id, data) = if data_length >= compression_threshold {
        let compressed_data = &slice[position..];
        let mut decoder = ZlibDecoder::new(compressed_data);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let mut position = 0;
        let mut slice = &decompressed_data[..];
        let packet_id = data::read_varint(&mut slice, &mut position)?;
        let data = slice[position..].to_vec();

        (packet_id, data)
    } else {
        // Uncompressed packet
        let packet_id = data::read_varint(&mut slice, &mut position)?;
        let data = slice[position..].to_vec();
        (packet_id, data)
    };

    Ok((packet_length, packet_id, data))
}

pub fn read_packet_from_bytes(
    mut slice: &[u8],
    compression_threshold: u32,
) -> Result<RawPacket, Error> {
    let mut position = 0;

    let packet_length = data::read_varint(&mut slice, &mut position)?;
    if slice.len() < packet_length as usize {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Packet too short"));
    }

    if compression_threshold == 0 {
        let packet_id = data::read_varint(&mut slice, &mut position)?;
        let data = slice[position..].to_vec();
        return Ok((packet_length, packet_id, data));
    }

    let data_length = data::read_varint(&mut slice, &mut position)?;

    let (packet_id, data) = if data_length >= compression_threshold {
        let compressed_data = &slice[position..];
        let mut decoder = ZlibDecoder::new(compressed_data);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let mut position = 0;
        let mut slice = &decompressed_data[..];
        let packet_id = data::read_varint(&mut slice, &mut position)?;
        let data = slice[position..].to_vec();

        (packet_id, data)
    } else {
        // Uncompressed packet
        let packet_id = data::read_varint(&mut slice, &mut position)?;
        let data = slice[position..].to_vec();
        (packet_id, data)
    };

    Ok((packet_length, packet_id, data))
}

pub(crate) async fn write_packet(
    packet: &dyn Packet,
    cnx: &mut TcpStream,
    compression_threshold: u32,
) -> Result<(), Error> {
    let mut buffer = Vec::new();

    // Write packet ID first
    let packet_id = packet.packet_id();
    data::write_varint(&mut buffer, packet_id);

    // Write packet data
    let mut data_buffer = Vec::new();
    packet.write_to(&mut data_buffer);

    // Concatenate ID + data
    buffer.extend(data_buffer);
    let uncompressed_length = buffer.len() as u32;

    let mut final_buffer = Vec::new();

    if compression_threshold == 0 {
        // No compression - use original format
        data::write_varint(&mut final_buffer, uncompressed_length);
        final_buffer.extend(buffer);
    } else if uncompressed_length >= compression_threshold {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&buffer)?;
        let compressed_data = encoder.finish()?;

        let total_compressed_length =
            compressed_data.len() as u32 + data::varint_size(uncompressed_length) as u32;

        data::write_varint(&mut final_buffer, total_compressed_length);
        data::write_varint(&mut final_buffer, uncompressed_length);

        final_buffer.extend(compressed_data);
    } else {
        let uncompressed_length_with_indicator = uncompressed_length + data::varint_size(0) as u32;

        data::write_varint(&mut final_buffer, uncompressed_length_with_indicator);
        data::write_varint(&mut final_buffer, 0);
        final_buffer.extend(buffer);
    }

    // Send packet
    cnx.write_all(&final_buffer).await?;
    cnx.flush().await?;

    Ok(())
}

pub trait Packet: Any + Send + Sync {
    fn id() -> u32
    where
        Self: Sized;

    fn packet_id(&self) -> u32; // This is a method to get the packet ID from an instance

    fn write_to(self: &Self, buffer: &mut Vec<u8>);

    fn read_from(
        id: u32,
        buffer: Vec<u8>,
    ) -> impl std::future::Future<Output = Result<Box<Self>, Box<std::io::Error>>> + Send
    where
        Self: Sized; // Desugared async function
}

pub(crate) fn downcast_packet<P: Packet>(
    packet: Arc<dyn Packet>,
) -> Result<Arc<P>, Box<std::io::Error>> {
    let anyed = packet as Arc<dyn Any + Send + Sync>;
    anyed.downcast::<P>().map_err(|_| {
        Box::new(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Failed to downcast packet to type: {}",
                std::any::type_name::<P>()
            ),
        ))
    })
}

// What an abomination this is, but it's nice
pub(crate) fn upcast_packet<E, P: Packet>(
    buffer: Result<Box<P>, E>,
) -> Result<Box<dyn Packet + 'static>, E> {
    buffer.map(|b| b as Box<dyn Packet + 'static>)
}

#[macro_export]
macro_rules! dispatch_packet_event {
    (
        packet = $packet:expr,
        server = $server:expr,
        state = $state:expr,
        connection = $connection:expr,
        table = {
            $(
                $conn_state:pat => [ $( $ty:ty ),* $(,)? ]
            ),* $(,)?
        }
    ) => {
        use crate::event::player_events::PlayerSentPacket;
        match $state {
            $(
                $conn_state => {
                    $(
                        if $packet.packet_id() == <$ty>::id() {
                            if let Ok(concrete) = Arc::downcast::<$ty>($packet) {
                                let event = Arc::new(PlayerSentPacket::<$ty> {
                                    packet: concrete,
                                    player_connection: $connection,
                                });

                                $server.lock().await.event_bus.dispatch(&event).await;

                            }
                        }
                    )*
                }
            )*
            _ => {
                eprintln!("Unknown state or packet: {:?} in state {:?}", $packet.packet_id(), $state);
            }
        }
    };
}
