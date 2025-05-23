use std::{
    io::ErrorKind,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio::{
    net::{TcpStream, tcp},
    sync::Mutex,
    task::JoinHandle,
};

use super::{Packet, PacketHeader};

pub struct ServerPacketManager {
    num_packets: Arc<AtomicU64>,
    pub stream_read: Arc<Mutex<tcp::OwnedReadHalf>>,
    pub stream_write: Arc<Mutex<tcp::OwnedWriteHalf>>,
}

impl ServerPacketManager {
    pub fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        Self {
            num_packets: Arc::new(AtomicU64::new(0)),
            stream_read: Arc::new(Mutex::new(read)),
            stream_write: Arc::new(Mutex::new(write)),
        }
    }

    pub fn start_listening<F>(&self, packet_handler: F) -> JoinHandle<()>
    where
        F: Fn(Packet) -> Packet + Send + 'static,
    {
        let stream_read = Arc::clone(&self.stream_read);
        let stream_write = Arc::clone(&self.stream_write);
        let num_packets = Arc::clone(&self.num_packets);

        tokio::spawn(async move {
            loop {
                let mut guard = stream_read.lock().await;
                match PacketHeader::read(&mut *guard).await {
                    Ok(packet) => {
                        let res = packet_handler(packet.packet);
                        let mut write_guard = stream_write.lock().await;
                        PacketHeader {
                            target_id: num_packets.fetch_add(1, Ordering::AcqRel) + 1,
                            packet: res,
                        }
                        .write(&mut *write_guard)
                        .await
                        .unwrap();
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::UnexpectedEof {
                            break;
                        } else {
                            eprintln!("Error receiving packet: {}", e);
                        }
                    }
                }
            }
        })
    }
}
