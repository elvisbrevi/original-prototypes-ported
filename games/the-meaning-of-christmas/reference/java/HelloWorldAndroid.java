package com.badlogic.gdx;

import android.os.Bundle;
import com.badlogic.gdx.backends.android.AndroidApplication;
import com.badlogic.gdx.helloworld.Juego;

/* JADX INFO: loaded from: classes.dex */
public class HelloWorldAndroid extends AndroidApplication {
    @Override // android.app.Activity
    public void onCreate(Bundle bundle) {
        super.onCreate(bundle);
        initialize((ApplicationListener) new Juego(), false);
    }
}
