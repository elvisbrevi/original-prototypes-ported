package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;

/* JADX INFO: loaded from: classes.dex */
public class Inicio {
    public Boolean activo = true;
    BitmapFont font;
    Texture imagen;
    Texture pixel;
    SpriteBatch spriteBatch;
    Texture titulo;

    public void cargar() {
        this.titulo = new Texture(Gdx.files.internal("data/titulo.png"));
        this.imagen = new Texture(Gdx.files.internal("data/imagen titulo.png"));
        this.pixel = new Texture(Gdx.files.internal("data/pixel.png"));
        this.spriteBatch = new SpriteBatch();
        this.font = new BitmapFont();
        this.font.setColor(Color.GREEN);
    }

    public void dibujar() {
        if (Gdx.input.justTouched()) {
            Operaciones.puntaje = 0;
            Operaciones.posicionDinero = -Operaciones.altoDinero;
            this.activo = false;
        }
        this.spriteBatch.begin();
        this.spriteBatch.draw(this.pixel, 0.0f, 0.0f, Gdx.graphics.getWidth(), Gdx.graphics.getHeight());
        this.spriteBatch.draw(this.titulo, 0.0f, Gdx.graphics.getHeight() - Operaciones.Porciento(30.0f, Gdx.graphics.getHeight()), Operaciones.Porciento(100.0f, Gdx.graphics.getWidth()), Operaciones.Porciento(30.0f, Gdx.graphics.getHeight()));
        this.spriteBatch.draw(this.imagen, 0.0f, Gdx.graphics.getHeight() - Operaciones.Porciento(110.0f, Gdx.graphics.getHeight()), Operaciones.Porciento(110.0f, Gdx.graphics.getWidth()), Operaciones.Porciento(100.0f, Gdx.graphics.getHeight()));
        if (Operaciones.puntaje > 0) {
            this.font.draw(this.spriteBatch, "puntaje: " + String.valueOf(Operaciones.puntaje) + ", haz click para empezar", 10.0f, 30.0f);
        } else {
            this.font.draw(this.spriteBatch, "haz click para empezar", 10.0f, 30.0f);
        }
        this.spriteBatch.end();
    }
}
