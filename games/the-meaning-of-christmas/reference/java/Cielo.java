package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;

/* JADX INFO: loaded from: classes.dex */
public class Cielo {
    Texture astro;
    Texture fondo;
    float anchoAstro = Operaciones.Porciento(30.0f, Gdx.graphics.getWidth());
    float altoAstro = Operaciones.Porciento(30.0f, Gdx.graphics.getHeight());

    public Cielo(String str) {
        if (str.contains("luna")) {
            this.astro = new Texture(Gdx.files.internal("data/luna.png"));
        }
        if (str.contains("noche")) {
            this.fondo = new Texture(Gdx.files.internal("data/noche.png"));
        }
    }

    public void dibujar(SpriteBatch spriteBatch) {
        spriteBatch.draw(this.fondo, 0.0f, 0.0f, Gdx.graphics.getWidth(), Gdx.graphics.getHeight());
        spriteBatch.draw(this.astro, Operaciones.Porciento(50.0f, Gdx.graphics.getWidth()), Operaciones.Porciento(70.0f, Gdx.graphics.getHeight()), this.anchoAstro, this.altoAstro);
    }
}
