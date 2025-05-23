use std::{collections::HashMap, io::ErrorKind, sync::Arc};

use tokio::{
    net::{TcpStream, tcp},
    sync::{Mutex, oneshot},
};

use super::{Packet, PacketHeader, PacketSendResult};

pub struct ClientPacketManager {
    num_packets: u64,
    pub stream_read: Arc<Mutex<tcp::OwnedReadHalf>>,
    pub stream_write: Arc<Mutex<tcp::OwnedWriteHalf>>,
    waiting_packets: Arc<Mutex<HashMap<u64, oneshot::Sender<Packet>>>>,
}

impl ClientPacketManager {
    pub fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        let this = Self {
            num_packets: 0,
            stream_read: Arc::new(Mutex::new(read)),
            stream_write: Arc::new(Mutex::new(write)),
            waiting_packets: Arc::new(Mutex::new(HashMap::new())),
        };
        this.start_listening();
        this
    }

    pub fn start_listening(&self) {
        let stream_read = Arc::clone(&self.stream_read);
        let waiting_packets = Arc::clone(&self.waiting_packets);
        tokio::spawn(async move {
            loop {
                let mut read_guard = stream_read.lock().await;
                match PacketHeader::read(&mut *read_guard).await {
                    Ok(packet) => {
                        if packet.target_id != 0 {
                            if let Some(sender) =
                                waiting_packets.lock().await.remove(&packet.target_id)
                            {
                                sender.send(packet.packet).unwrap_or_else(|_| {
                                    eprintln!("Failed to send packet result");
                                });
                            } else {
                                eprintln!("No sender found for packet ID: {}", packet.target_id);
                            }
                        } else {
                            eprintln!("Invalid packet recieved: {:?}", packet.packet_type());
                        }
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
        });
    }

    pub async fn send_packet(
        &mut self,
        packet: &PacketHeader,
    ) -> std::io::Result<PacketSendResult> {
        let mut guard = self.stream_write.lock().await;
        packet.write(&mut *guard).await?;
        self.num_packets += 1;
        // Create a oneshot channel to receive the response
        let (tx, rx) = oneshot::channel();
        self.waiting_packets
            .lock()
            .await
            .insert(self.num_packets, tx);
        Ok(PacketSendResult {
            id: self.num_packets,
            result: rx,
        })
    }
}
