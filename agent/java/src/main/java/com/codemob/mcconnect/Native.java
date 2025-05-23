package com.codemob.mcconnect;

public class Native {
    public static void load(String lib) {
        System.load(lib);
    }

    public static native void init();
}