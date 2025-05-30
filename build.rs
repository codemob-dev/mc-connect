use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Path to the Gradle project
    let agent_dir = PathBuf::from("agent/java");

    // Run `./gradlew build` or `gradle build` in src/agent
    let status = Command::new("./gradlew")
        .arg("build")
        .current_dir(&agent_dir)
        .status()
        .expect("Failed to run Gradle build");

    if !status.success() {
        panic!("Gradle build failed");
    }

    // Path to the built .jar (adjust if your jar name is different)
    let jar_path = agent_dir
        .join("build/libs")
        .read_dir()
        .expect("Failed to read libs directory")
        .find_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "jar" {
                Some(path)
            } else {
                None
            }
        })
        .expect("No JAR file found in build/libs");

    println!("cargo:rustc-env=AGENT_JAR={}", jar_path.display());
    println!("cargo:rerun-if-changed=src/agent");
}
