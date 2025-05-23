use std::{
    env::current_exe,
    fs::{self, File, set_permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    thread::{self, sleep},
    time::Duration,
};

use itertools::Itertools;
use jni::{InitArgsBuilder, JavaVM};
use sysinfo::{Process, System};
use tokio::net::TcpStream;

use crate::communication::{ADDRESS, client::ClientPacketManager};

const AGENT_JAR: &[u8] = include_bytes!("agent/agent.jar");

fn write_agent_jar(dir: &Path) -> std::io::Result<String> {
    let path = dir.join("agent.jar");
    // Write to a temporary file
    let mut file = File::create(&path)?;
    file.write_all(AGENT_JAR)?;

    // Set permissions to readable by everyone (rw-r--r--)
    let perms = PermissionsExt::from_mode(0o644);
    set_permissions(&path, perms)?;

    // Persist the file so it doesn't get deleted
    Ok(path.display().to_string())
}

pub struct MinecraftInstance {
    pub version: String,
    pub packet_manager: ClientPacketManager,
}

impl MinecraftInstance {
    pub async fn load(process: &Process, pid: &str, version: String) -> jni::errors::Result<Self> {
        let agent_jar =
            write_agent_jar(process.cwd().unwrap()).expect("Failed to write embedded agent JAR");

        let lib = current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("librust_agent.so");
        let out = process.cwd().unwrap().join("librust_agent.so");
        fs::copy(lib, &out).expect("Failed to copy library");

        let current_thread = thread::current();

        let pid = pid.to_string();
        thread::spawn(move || {
            let jvm = load_jvm();
            let mut env = jvm.attach_current_thread().unwrap();

            // Find the VirtualMachine class
            let vm_class = env
                .find_class("com/sun/tools/attach/VirtualMachine")
                .unwrap();

            // Convert pid & agent_path to JNI strings
            let pid_jstring = env.new_string(pid).unwrap();

            // Call static VirtualMachine.attach(String pid)
            let vm_obj = env
                .call_static_method(
                    vm_class,
                    "attach",
                    "(Ljava/lang/String;)Lcom/sun/tools/attach/VirtualMachine;",
                    &[(&pid_jstring).into()],
                )
                .unwrap()
                .l()
                .unwrap();

            let arg = env.new_string(out.to_str().unwrap()).unwrap();
            let agent_jstring = env.new_string(agent_jar).unwrap();

            current_thread.unpark();
            let res = env.call_method(
                &vm_obj,
                "loadAgent",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[(&agent_jstring).into(), (&arg).into()],
            );

            if let Err(e) = res {
                eprintln!("Failed to load agent: {}", e);
            }
            env.call_method(&vm_obj, "detach", "()V", &[])
                .map(|_| ())
                .unwrap_or_else(|e| eprintln!("Failed to detach: {}", e));
        });
        thread::park();

        println!("Attempting to connect to agent at {}", ADDRESS);
        let mut stream = TcpStream::connect(ADDRESS).await;
        for _ in 0..10 {
            if stream.is_ok() {
                break;
            }
            sleep(Duration::from_secs(1));
            stream = TcpStream::connect(ADDRESS).await;
        }
        let stream = stream.unwrap();
        println!("Connected to agent at {}", ADDRESS);

        Ok(Self {
            packet_manager: ClientPacketManager::new(stream),
            version,
        })
    }
}

fn get_mc_version(process: &Process) -> Option<String> {
    let idx = process.cmd().iter().position(|arg| arg == "-cp")?;
    let classpath = &process.cmd()[idx + 1];
    let mc_path = Path::new(classpath.to_str().unwrap().split(':').last()?);
    let jar_file = mc_path.file_name()?.to_str()?;
    Some(
        jar_file
            .strip_prefix("minecraft-")?
            .strip_suffix("-client.jar")?
            .to_string(),
    )
}

fn load_jvm() -> JavaVM {
    let jvm_args = InitArgsBuilder::new()
        .version(jni::JNIVersion::V8)
        .option("--add-modules=jdk.attach")
        .build()
        .unwrap();

    JavaVM::new(jvm_args).unwrap()
}

pub async fn find_and_connect() -> MinecraftInstance {
    let s = System::new_all();
    use std::ffi::OsStr;
    let mut processes = s
        .processes_by_name(OsStr::new("java"))
        .filter_map(|process| {
            get_mc_version(process).map(|mc_version| (process, process.pid(), mc_version))
        })
        .collect_vec();

    processes.sort_by_key(|(process, _, _)| process.start_time());
    let (process, pid, version) = processes
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("No Minecraft process found"));
    println!("Found Minecraft version: {}\nPID: {}", version, pid);
    MinecraftInstance::load(process, &pid.to_string(), version.clone())
        .await
        .unwrap()
}
