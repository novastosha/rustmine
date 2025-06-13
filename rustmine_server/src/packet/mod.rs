use std::{
    any::Any,
    io::{Error, ErrorKind, Read, Write},
    sync::Arc,
};

use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use rustmine_lib::data;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod clientbound;
pub mod serverbound;

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

#[macro_export]
macro_rules! packet_id {
    // Generate the function to get the packet ID
    ($id:literal) => {
        fn id() -> u32
        where
            Self: Sized,
        {
            $id
        }

        fn packet_id(&self) -> u32 {
            Self::id()
        }
    };
}

#[macro_export]
macro_rules! id_match {
    ($id:expr, $expected:expr) => {
        if $id != $expected {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Packet ID mismatch: expected {}, got {}", $expected, $id),
            )));
        }
    };
}

#[macro_export]
macro_rules! clientbound_packet {
    () => {
        async fn read_from(
            _: u32,
            _: Vec<u8>,
        ) -> Result<Box<Self>, Box<std::io::Error>> {
            unimplemented!(
                concat!(
                    stringify!($crate::packet::Packet),
                    " is a clientbound packet and should not be read from a buffer"
                )
            );
        }
    };
}

#[macro_export]
macro_rules! serverbound_packet {
    () => {
        fn write_to(&self, _buffer: &mut Vec<u8>) {
            unimplemented!(
                concat!(
                    stringify!($crate::packet::Packet),
                    " is a serverbound packet and should not be written to a buffer"
                )
            );
        }
    };
}
