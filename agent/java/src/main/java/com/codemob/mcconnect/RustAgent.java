package com.codemob.mcconnect;

import java.lang.instrument.Instrumentation;
import java.util.Arrays;

public class RustAgent {
    public static YarnMappingResolver mappingResolver;
    public static String version;
    public static ClassLoader classLoader;
    public static void agentmain(String agentArgs, Instrumentation inst) throws Exception {
        classLoader = Arrays.stream(inst.getAllLoadedClasses())
                .filter(cls -> cls.toString().contains("net.minecraft"))
                .findFirst()
                .orElseThrow()
                .getClassLoader();
        version = MinecraftVersionResolver.resolveVersion();
        mappingResolver = new YarnMappingResolver(version);
        Native.load(agentArgs);
        Native.init();
    }
}