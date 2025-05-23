// src/lib.rs
pub mod communication;

use communication::server::ServerPacketManager;
use communication::{ADDRESS, Packet};

use jni::objects::JClass;
use jni::{JNIEnv, JavaVM};
use tokio::net::TcpListener;

#[tokio::main]
#[unsafe(no_mangle)]
pub async extern "system" fn Java_com_codemob_Native_init(env: JNIEnv<'_>, _class: JClass<'_>) {
    let vm: JavaVM = env.get_java_vm().unwrap();
    let _ = tokio::spawn(async move {
        let listener = TcpListener::bind(ADDRESS)
            .await
            .expect("Failed to bind TCP listener");
        println!("Listening on {}", ADDRESS);
        let (socket, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        println!("Accepted connection from {}", addr);
        let packet_manager = ServerPacketManager::new(socket);
        packet_manager
            .start_listening(move |packet| {
                let mut env = vm.attach_current_thread_as_daemon().unwrap();
                let tools_class = env.find_class("com/codemob/Tools").unwrap();

                match packet {
                    Packet::Print(print_packet) => {
                        print!("{}", print_packet.message);
                        Packet::Confirmation
                    }
                    Packet::Toast(toast_packet) => {
                        env.call_static_method(
                            tools_class,
                            "showToast",
                            "(Ljava/lang/String;Ljava/lang/String;)V",
                            &[
                                (&env.new_string(toast_packet.title).unwrap()).into(),
                                (&env.new_string(toast_packet.body).unwrap()).into(),
                            ],
                        )
                        .unwrap();
                        Packet::Confirmation
                    }
                    packet => {
                        eprintln!("Invalid packet type recieved: {:?}", packet.packet_type());
                        Packet::Err
                    }
                }
            })
            .await
            .unwrap();
        println!("Connection closed.");
    })
    .await;
}
