package com.codemob;

import java.lang.reflect.Method;
import java.time.Duration;
import java.util.Arrays;

public class Tools {
    public static Class<?> loadClass(String className) throws ClassNotFoundException {
        String mappedName = RustAgent.mappingResolver.getClassMapping(className).replace('/', '.');
        return RustAgent.classLoader.loadClass(mappedName);
    }
    public static void showToast(String title, String description) {
        try {
            Class<?> minecraftClient = loadClass("net/minecraft/client/MinecraftClient");
            Class<?> systemToast = loadClass("net/minecraft/client/toast/SystemToast");
            Class<?> toastManager = loadClass("net/minecraft/client/toast/ToastManager");
            Class<?> systemToast$type = loadClass("net/minecraft/client/toast/SystemToast$Type");
            Class<?> text = loadClass("net/minecraft/text/Text");


            Method text$of = text.getMethod(RustAgent.mappingResolver.getMethodMapping(
                    "net/minecraft/text/Text",
                    "of",
                    "(Ljava/lang/String;)Lnet/minecraft/text/Text;"
            ), String.class);

            Object client = minecraftClient.getMethod(RustAgent.mappingResolver.getMethodMapping(
                    "net/minecraft/client/MinecraftClient",
                    "getInstance",
                    "()Lnet/minecraft/client/MinecraftClient;")).invoke(null);

            Object manager = minecraftClient.getMethod(RustAgent.mappingResolver.getMethodMapping(
                    "net/minecraft/client/MinecraftClient",
                    "getToastManager",
                    "()Lnet/minecraft/client/toast/ToastManager;")).invoke(client);

            Object titleText = text$of.invoke(null, title);
            Object descriptionText = text$of.invoke(null, description);

            Object textType = systemToast$type.getField(RustAgent.mappingResolver.getFieldMapping(
                    "net/minecraft/client/toast/SystemToast$Type",
                    "PERIODIC_NOTIFICATION",
                    "Lnet/minecraft/client/toast/SystemToast$Type;"
                    )).get(null);

            systemToast.getMethod(RustAgent.mappingResolver.getMethodMapping(
                    "net/minecraft/client/toast/SystemToast",
                    "show",
                    "(Lnet/minecraft/client/toast/ToastManager;Lnet/minecraft/client/toast/SystemToast$Type;Lnet/minecraft/text/Text;Lnet/minecraft/text/Text;)V"),
                    toastManager, systemToast$type, text, text
                    ).invoke(null, manager, textType, titleText, descriptionText);

        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }
}
