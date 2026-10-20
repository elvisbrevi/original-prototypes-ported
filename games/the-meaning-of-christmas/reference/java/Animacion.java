package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.Animation;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.badlogic.gdx.graphics.g2d.TextureRegion;

/* JADX INFO: loaded from: classes.dex */
public class Animacion {
    private static final int columnas = 4;
    private static final int filas = 1;
    float alto;
    float ancho;
    Animation animacion;
    TextureRegion framaActual;
    TextureRegion[] framas;
    float stateTime;
    Texture textura;
    float velocidad = 1.0f;

    public void cargar(String str, float f, float f2) {
        setAncho(Operaciones.Porciento(f, Gdx.graphics.getWidth()));
        setAlto(Operaciones.Porciento(f2, Gdx.graphics.getHeight()));
        this.textura = new Texture(Gdx.files.internal(str));
        TextureRegion[][] textureRegionArrSplit = TextureRegion.split(this.textura, this.textura.getWidth() / 4, this.textura.getHeight() / 1);
        this.framas = new TextureRegion[4];
        int i = 0;
        int i2 = 0;
        while (i2 < 1) {
            int i3 = 0;
            int i4 = i;
            while (i3 < 4) {
                this.framas[i4] = textureRegionArrSplit[i2][i3];
                i3++;
                i4++;
            }
            i2++;
            i = i4;
        }
        this.animacion = new Animation(0.025f, this.framas);
        this.stateTime = 0.0f;
    }

    public void dibujar(SpriteBatch spriteBatch, float f, float f2) {
        this.stateTime += Gdx.graphics.getDeltaTime() * getVelocidad();
        this.framaActual = this.animacion.getKeyFrame(this.stateTime, true);
        spriteBatch.draw(this.framaActual, f, f2, getAncho(), getAlto());
    }

    public float getAlto() {
        return this.alto;
    }

    public float getAncho() {
        return this.ancho;
    }

    public float getVelocidad() {
        return this.velocidad;
    }

    public void setAlto(float f) {
        this.alto = f;
    }

    public void setAncho(float f) {
        this.ancho = f;
    }

    public void setVelocidad(float f) {
        this.velocidad = f;
    }
}
