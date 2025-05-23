use std::{
    env, io,
    process::{self, Command},
};

use itertools::Itertools;

use crate::{
    communication::{PacketSendResult, PrintPacket, RunPacket, SendableCommand, ToastPacket},
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

    pub async fn run(
        &mut self,
        command: impl Into<SendableCommand>,
    ) -> io::Result<PacketSendResult> {
        let packet = RunPacket::new(command);
        self.packet_manager.send_packet(&packet.as_header()).await
    }

    pub async fn restart_in_mc(&mut self) -> std::io::Result<()> {
        let mut cmd = recreate_command()?;
        cmd.env("IN_MC", "true");
        self.run(cmd);
        process::exit(0)
    }
}

fn recreate_command() -> std::io::Result<Command> {
    let exe = env::current_exe()?;
    let args = env::args_os().skip(1).collect_vec();
    let mut cmd = Command::new(exe);

    cmd.args(args);

    for (key, value) in env::vars_os() {
        cmd.env(key, value);
    }

    cmd.current_dir(env::current_dir()?);

    Ok(cmd)
}
