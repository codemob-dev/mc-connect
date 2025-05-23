plugins {
    id("java")
    id("com.github.johnrengelman.shadow") version "7.1.2"
}
group = "com.codemob.mcconnect"
version = "1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation("net.fabricmc:mapping-io:0.6.1")
    implementation("org.jetbrains:annotations:15.0")
}

tasks.shadowJar {
    archiveClassifier.set("") // makes it overwrite the default JAR
    mergeServiceFiles()
    manifest {
        attributes(
            "Agent-Class" to "com.codemob.mcconnect.RustAgent",
            "Can-Redefine-Classes" to "true",
            "Can-Retransform-Classes" to "true"
        )
    }
}

// Disable default JAR task so only the shaded one is produced
tasks.named<Jar>("jar") {
    enabled = false
}

// Make build task use shadowJar instead
tasks.named("build") {
    dependsOn(tasks.named("shadowJar"))
}