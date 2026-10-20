package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.backends.jogl.JoglApplication;

/* JADX INFO: loaded from: classes.dex */
public class JuegoEscritorio {
    public static JoglApplication app;

    public static void main(String[] strArr) {
        new JoglApplication(new Juego(), "The Meaning of Christmas", 240, 320, false);
    }
}
