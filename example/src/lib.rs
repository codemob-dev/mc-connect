use jni::JavaVM;
use mc_connect::minecraft::MinecraftContext;

#[unsafe(no_mangle)]
pub extern "C" fn in_mc(jvm: &JavaVM) {
    println!("Hello minecraft!");
    let context = MinecraftContext::from_jvm(jvm).unwrap();
    println!("Minecraft version: {}", context.version);
}
