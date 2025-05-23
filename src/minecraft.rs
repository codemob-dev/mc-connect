use std::io;

use crate::{
    communication::{PacketSendResult, PrintPacket, ToastPacket},
    initialization::MinecraftInstance,
};

impl MinecraftInstance {
    pub async fn print(&mut self, message: &str) -> io::Result<PacketSendResult> {
        let packet = PrintPacket::new(message.to_string());
        self.packet_manager.send_packet(&packet.as_header()).await
    }

    pub async fn println(&mut self, message: &str) -> io::Result<PacketSendResult> {
        let packet = PrintPacket::new(message.to_string() + "\n");
        self.packet_manager.send_packet(&packet.as_header()).await
    }

    pub async fn toast(&mut self, title: &str, body: &str) -> io::Result<PacketSendResult> {
        let packet = ToastPacket::new(title.to_string(), body.to_string());
        self.packet_manager.send_packet(&packet.as_header()).await
    }
}
