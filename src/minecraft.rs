use std::{fmt, fs, io, path::PathBuf};

use jni::{AttachGuard, JavaVM, objects::JClass};

use crate::{
    communication::{PacketSendResult, PrintPacket, RunPacket, ToastPacket},
    initialization::MinecraftProcess,
};

impl MinecraftProcess {
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

    pub async fn run(&mut self, lib: PathBuf, func: String) -> io::Result<PacketSendResult> {
        let new_loc = self.dotminecraft.join(lib.file_name().unwrap());
        fs::copy(lib, &new_loc)?;
        let packet = RunPacket::new(new_loc, func);
        self.packet_manager.send_packet(&packet.as_header()).await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContextLoadError<T: fmt::Display>(T);

impl<T: fmt::Display> fmt::Display for ContextLoadError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "minecraft context loading failed: {}", self.0)
    }
}

pub struct MinecraftContext<'a> {
    pub env: AttachGuard<'a>,
    pub agent_class: JClass<'a>,
    pub version: String,
}

impl<'a> MinecraftContext<'a> {
    pub fn from_jvm(jvm: &'a JavaVM) -> Result<Self, ContextLoadError<anyhow::Error>> {
        let mut env = jvm.attach_current_thread().unwrap();
        let agent_class = env
            .find_class("com/codemob/mcconnect/RustAgent")
            .map_err(|e| ContextLoadError(e.into()))?;
        let version_str = env
            .get_static_field(&agent_class, "version", "Ljava/lang/String;")
            .map_err(|e| ContextLoadError(e.into()))?
            .l() // Get the jobject
            .map_err(|e| ContextLoadError(e.into()))?;
        let version = env
            .get_string((&version_str).into())
            .map_err(|e| ContextLoadError(e.into()))?
            .to_str()
            .map_err(|e| ContextLoadError(e.into()))?
            .to_owned();

        Ok(MinecraftContext {
            env,
            version,
            agent_class,
        })
    }
}
