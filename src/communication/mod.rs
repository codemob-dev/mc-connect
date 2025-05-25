pub mod client;
pub mod server;

use std::path::PathBuf;

use bincode::{Decode, Encode};
use futures::{SinkExt, StreamExt};
use tokio::sync::oneshot;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub const ADDRESS: &str = "127.0.0.1:8080";

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct PacketHeader {
    target_id: u64,
    packet: Packet,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum Packet {
    Print(PrintPacket),
    Toast(ToastPacket),
    Invoke(InvokePacket),
    Run(RunPacket),
    Confirmation,
    Err,
}

pub struct PacketSendResult {
    pub id: u64,
    result: oneshot::Receiver<Packet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct PrintPacket {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ToastPacket {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct InvokePacket {
    pub class_name: String,
    pub method_name: String,
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct RunPacket {
    pub lib: PathBuf,
    pub func: String,
}

impl PacketSendResult {
    pub async fn get_result(self) -> Packet {
        self.result.await.unwrap()
    }
}

impl Packet {
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
        let bytes = reader
            .next()
            .await
            .ok_or(std::io::ErrorKind::UnexpectedEof)??;

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

impl RunPacket {
    pub fn new(lib: PathBuf, func: String) -> Packet {
        Packet::Run(Self { lib, func })
    }
}
