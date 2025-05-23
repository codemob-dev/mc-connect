pub mod client;
pub mod server;

use std::str;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::oneshot;

pub const ADDRESS: &str = "127.0.0.1:8080";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum PacketType {
    PRINT,
    TOAST,
    CONFIRMATION,
    ERR,
}

#[derive(Debug, Clone)]
pub struct PacketHeader {
    target_id: u64,
    packet: Packet,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Packet {
    Print(PrintPacket),
    Toast(ToastPacket),
    Confirmation,
    Err,
}

pub struct PacketSendResult {
    pub id: u64,
    result: oneshot::Receiver<Packet>,
}

impl PacketSendResult {
    pub async fn get_result(self) -> Packet {
        self.result.await.unwrap()
    }
}

impl Packet {
    pub fn packet_type(&self) -> PacketType {
        match self {
            Packet::Print(_) => PacketType::PRINT,
            Packet::Toast(_) => PacketType::TOAST,
            Packet::Confirmation => PacketType::CONFIRMATION,
            Packet::Err => PacketType::ERR,
        }
    }

    pub fn as_header(self) -> PacketHeader {
        PacketHeader {
            target_id: 0,
            packet: self,
        }
    }

    pub fn as_response(self, target_id: u64) -> PacketHeader {
        PacketHeader {
            target_id,
            packet: self,
        }
    }
}

impl PacketHeader {
    pub fn packet_type(&self) -> PacketType {
        self.packet.packet_type()
    }

    pub async fn write<T>(&self, output: &mut T) -> std::io::Result<()>
    where
        T: tokio::io::AsyncWriteExt + Unpin,
    {
        output.write_u8(self.packet_type().into()).await?;
        output.write_u64(self.target_id).await?;
        match &self.packet {
            Packet::Print(packet) => packet.write(output).await?,
            Packet::Toast(packet) => packet.write(output).await?,
            _ => {} // empty packets
        }
        output.flush().await
    }

    pub async fn read<T>(input: &mut T) -> std::io::Result<Self>
    where
        Self: Sized,
        T: tokio::io::AsyncReadExt + Unpin,
    {
        let packet_type = input.read_u8().await?;
        let target_id = input.read_u64().await?;
        match PacketType::try_from(packet_type) {
            Ok(PacketType::PRINT) => PrintPacket::read(input).await.map(Packet::Print),
            Ok(PacketType::TOAST) => ToastPacket::read(input).await.map(Packet::Toast),
            Ok(PacketType::CONFIRMATION) => Ok(Packet::Confirmation),
            Ok(PacketType::ERR) => Ok(Packet::Err),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown packet type",
            )),
        }
        .map(|packet| PacketHeader { target_id, packet })
    }
}

trait Packetable {
    async fn write<T>(&self, output: &mut T) -> std::io::Result<()>
    where
        T: tokio::io::AsyncWriteExt + Unpin;
    async fn read<T>(input: &mut T) -> std::io::Result<Self>
    where
        Self: Sized,
        T: tokio::io::AsyncReadExt + Unpin;
}

#[derive(Debug, Clone)]
pub struct PrintPacket {
    pub message: String,
}

impl PrintPacket {
    pub fn new(message: String) -> Packet {
        Packet::Print(PrintPacket { message })
    }
}

impl Packetable for PrintPacket {
    async fn write<T>(&self, output: &mut T) -> std::io::Result<()>
    where
        T: tokio::io::AsyncWriteExt + Unpin,
    {
        let message_bytes = self.message.as_bytes();
        let length = message_bytes.len() as u32;
        output.write_u32(length).await?;
        output.write_all(message_bytes).await?;
        Ok(())
    }

    async fn read<T>(input: &mut T) -> std::io::Result<Self>
    where
        Self: Sized,
        T: tokio::io::AsyncReadExt + Unpin,
    {
        let length = input.read_u32().await?;

        let mut message_buf = vec![0u8; length as usize];
        input.read_exact(&mut message_buf).await?;

        let message = String::from_utf8(message_buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        Ok(PrintPacket { message })
    }
}

#[derive(Debug, Clone)]
pub struct ToastPacket {
    pub title: String,
    pub body: String,
}

impl ToastPacket {
    pub fn new(title: String, body: String) -> Packet {
        Packet::Toast(ToastPacket { title, body })
    }
}

impl Packetable for ToastPacket {
    async fn write<T>(&self, output: &mut T) -> std::io::Result<()>
    where
        T: tokio::io::AsyncWriteExt + Unpin,
    {
        let title_bytes = self.title.as_bytes();
        let body_bytes = self.body.as_bytes();
        let title_length = title_bytes.len() as u32;
        let body_length = body_bytes.len() as u32;

        output.write_u32(title_length).await?;
        output.write_all(title_bytes).await?;
        output.write_u32(body_length).await?;
        output.write_all(body_bytes).await?;

        Ok(())
    }

    async fn read<T>(input: &mut T) -> std::io::Result<Self>
    where
        Self: Sized,
        T: tokio::io::AsyncReadExt + Unpin,
    {
        let title_length = input.read_u32().await?;

        let mut title_buf = vec![0u8; title_length as usize];
        input.read_exact(&mut title_buf).await?;

        let title = String::from_utf8(title_buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        let body_length = input.read_u32().await?;

        let mut body_buf = vec![0u8; body_length as usize];
        input.read_exact(&mut body_buf).await?;

        let body = String::from_utf8(body_buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        Ok(ToastPacket { title, body })
    }
}
