package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.ApplicationListener;
import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.audio.Music;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.badlogic.gdx.math.Vector2;
import java.util.ArrayList;

/* JADX INFO: loaded from: classes.dex */
public class Juego implements ApplicationListener {
    float anchoDinero;
    int anchoFuego;
    Music angel;
    Texture barra;
    Cielo cielo;
    Texture dinero;
    BitmapFont font;
    Inicio inicio;
    Music noche;
    Vector2 posicionDinero;
    SpriteBatch spriteBatch;
    ArrayList<Regalo> regalos = new ArrayList<>();
    Animacion animacionFuego = new Animacion();

    @Override // com.badlogic.gdx.ApplicationListener
    public void create() {
        this.noche = Gdx.audio.newMusic(Gdx.files.internal("data/noche de paz.mp3"));
        this.angel = Gdx.audio.newMusic(Gdx.files.internal("data/angel.mp3"));
        this.inicio = new Inicio();
        this.inicio.cargar();
        this.barra = new Texture(Gdx.files.internal("data/pixel.png"));
        this.dinero = new Texture(Gdx.files.internal("data/dinero.png"));
        this.anchoDinero = Gdx.graphics.getWidth();
        Operaciones.altoDinero = Gdx.graphics.getHeight();
        this.posicionDinero = new Vector2(0.0f, 0.0f);
        Operaciones.posicionDinero = -Operaciones.altoDinero;
        this.animacionFuego.cargar("data/franja.png", 8.0f, 8.0f);
        this.animacionFuego.setVelocidad(0.1f);
        this.anchoFuego = (int) (Gdx.graphics.getWidth() / this.animacionFuego.getAncho());
        this.cielo = new Cielo("luna noche");
        this.font = new BitmapFont();
        this.font.setColor(Color.BLACK);
        int i = 1;
        while (i <= 9) {
            Regalo regalo = i <= 3 ? new Regalo(1) : null;
            if (i > 3 && i <= 6) {
                regalo = new Regalo(2);
            }
            if (i > 6 && i <= 9) {
                regalo = new Regalo(3);
            }
            regalo.setPosicion(0, 0);
            this.regalos.add(regalo);
            i++;
        }
        this.spriteBatch = new SpriteBatch();
    }

    @Override // com.badlogic.gdx.ApplicationListener
    public void dispose() {
    }

    @Override // com.badlogic.gdx.ApplicationListener
    public void pause() {
    }

    @Override // com.badlogic.gdx.ApplicationListener
    public void render() {
        if (this.inicio.activo.booleanValue()) {
            if (this.angel.isPlaying()) {
                this.angel.stop();
            }
            if (!this.noche.isPlaying()) {
                this.noche.play();
            }
            this.inicio.dibujar();
            return;
        }
        if (Operaciones.posicionDinero >= (-Operaciones.Porciento(10.0f, Gdx.graphics.getHeight()))) {
            this.inicio.activo = true;
        }
        if (this.noche.isPlaying()) {
            this.noche.stop();
        }
        if (!this.angel.isPlaying()) {
            this.angel.play();
        }
        for (int i = 0; i < this.regalos.size(); i++) {
            this.regalos.get(i).actualizar(this.animacionFuego, this.inicio.activo.booleanValue());
        }
        if (Operaciones.error.booleanValue()) {
            for (int i2 = 0; i2 < this.regalos.size(); i2++) {
                this.regalos.get(i2).setColor(Color.WHITE);
                this.regalos.get(i2).setSeleccionado(false);
            }
            Operaciones.error = false;
        }
        Operaciones.controlarCaida(this.regalos);
        this.spriteBatch.begin();
        this.cielo.dibujar(this.spriteBatch);
        if (Operaciones.cantidadSeleccionados > 0) {
            for (int i3 = 0; i3 <= this.anchoFuego; i3++) {
                this.animacionFuego.dibujar(this.spriteBatch, (i3 * this.animacionFuego.getAncho()) - (this.animacionFuego.getAncho() / 2.0f), Operaciones.posicionFranja);
            }
        }
        for (int i4 = 0; i4 < this.regalos.size(); i4++) {
            if (Operaciones.cantidadSeleccionados >= 3) {
                for (int i5 = 0; i5 < this.regalos.size(); i5++) {
                    this.regalos.get(i5).setSeleccionado(false);
                }
                Operaciones.cantidadSeleccionados = 0;
            }
            this.regalos.get(i4).dibujar(this.spriteBatch);
        }
        this.spriteBatch.draw(this.dinero, this.posicionDinero.x, Operaciones.posicionDinero, this.anchoDinero, Operaciones.altoDinero);
        this.spriteBatch.setColor(Color.WHITE);
        this.spriteBatch.draw(this.barra, 0.0f, Gdx.graphics.getHeight() - Operaciones.Porciento(8.0f, Gdx.graphics.getHeight()), Gdx.graphics.getWidth(), Operaciones.Porciento(8.0f, Gdx.graphics.getHeight()));
        this.font.draw(this.spriteBatch, "Puntaje: " + String.valueOf(Operaciones.puntaje), 10.0f, Gdx.graphics.getHeight() - Operaciones.Porciento(2.0f, Gdx.graphics.getHeight()));
        this.spriteBatch.setColor(Color.WHITE);
        this.spriteBatch.end();
    }

    @Override // com.badlogic.gdx.ApplicationListener
    public void resize(int i, int i2) {
        this.spriteBatch.getProjectionMatrix().setToOrtho2D(0.0f, 0.0f, i, i2);
    }

    @Override // com.badlogic.gdx.ApplicationListener
    public void resume() {
    }
}
