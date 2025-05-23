package com.codemob.mcconnect;

import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class MinecraftVersionResolver {
    public static String resolveVersion() {
        // We haven't initialized mappings yet, so we have to manually put in the unmapped names.
        try {
            System.out.println("Finding minecraft version...");
            // net.minecraft.MinecraftVersion

            Class<?> minecraft_version = RustAgent.classLoader.loadClass("net.minecraft.class_3797");
            // MinecraftVersion.CURRENT
            Field current = minecraft_version.getField("field_25319");
            // net.minecraft.GameVersion
            Object game_version = current.get(null);
            // String GameVersion.getName()
            Method get_name = game_version.getClass().getMethod("method_48019");
            String version = (String) get_name.invoke(game_version);
            System.out.println("Found version " + version);
            return version;
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }
}
