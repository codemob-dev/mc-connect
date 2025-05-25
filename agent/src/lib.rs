use libloading::{Library, Symbol};
use mc_connect::communication::server::ServerPacketManager;
use mc_connect::communication::{ADDRESS, Packet};

use jni::objects::JClass;
use jni::{JNIEnv, JavaVM};
use tokio::net::TcpListener;

#[tokio::main]
#[unsafe(no_mangle)]
pub async extern "system" fn Java_com_codemob_mcconnect_Native_init(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
) {
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
                let tools_class = env.find_class("com/codemob/mcconnect/Tools").unwrap();

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
                    Packet::Run(packet) => unsafe {
                        let lib = Library::new(packet.lib).unwrap();
                        let func: Symbol<unsafe extern "C" fn(&JavaVM)> =
                            lib.get(packet.func.as_bytes()).unwrap();
                        func(&vm);
                        Packet::Confirmation
                    },
                    packet => {
                        eprintln!("Invalid packet recieved: {:?}", packet);
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
