pub mod client;
pub mod server;

use std::str;

use bincode::{Decode, Encode};
use futures::{SinkExt, StreamExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::oneshot;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub const ADDRESS: &str = "127.0.0.1:8080";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum PacketType {
    PRINT,
    TOAST,
    INVOKE,
    CONFIRMATION,
    ERR,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PacketHeader {
    target_id: u64,
    packet: Packet,
}

#[repr(u8)]
#[derive(Debug, Clone, Encode, Decode)]
pub enum Packet {
    Print(PrintPacket),
    Toast(ToastPacket),
    Invoke(InvokePacket),
    Confirmation,
    Err,
}

pub struct PacketSendResult {
    pub id: u64,
    result: oneshot::Receiver<Packet>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PrintPacket {
    pub message: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ToastPacket {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct InvokePacket {
    pub class_name: String,
    pub method_name: String,
    pub desc: String,
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
            Packet::Invoke(_) => PacketType::INVOKE,
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
        let mut writer = FramedWrite::new(output, LengthDelimitedCodec::new());

        let res = bincode::encode_to_vec(self, bincode::config::standard()).unwrap();
        writer.send(res.into()).await
    }

    pub async fn read<T>(input: &mut T) -> std::io::Result<Self>
    where
        Self: Sized,
        T: tokio::io::AsyncReadExt + Unpin,
    {
        let mut reader = FramedRead::new(input, LengthDelimitedCodec::new());
        let bytes = reader.next().await.unwrap()?;

        Ok(
            bincode::decode_from_slice(&bytes, bincode::config::standard())
                .unwrap()
                .0,
        )
    }
}

impl PrintPacket {
    pub fn new(message: String) -> Packet {
        Packet::Print(PrintPacket { message })
    }
}

impl ToastPacket {
    pub fn new(title: String, body: String) -> Packet {
        Packet::Toast(ToastPacket { title, body })
    }
}
