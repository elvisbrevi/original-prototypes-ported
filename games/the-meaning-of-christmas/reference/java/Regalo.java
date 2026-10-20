package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.badlogic.gdx.math.Vector2;

/* JADX INFO: loaded from: classes.dex */
public class Regalo {
    Texture textura;
    Vector2 posicion = new Vector2();
    Boolean seleccionado = false;
    Color color = Color.WHITE;
    Boolean esperar = false;
    float ancho = Operaciones.Porciento(10.0f, Gdx.graphics.getWidth());
    float alto = Operaciones.Porciento(10.0f, Gdx.graphics.getHeight());
    float velocidad = 1.0f;

    public Regalo(int i) {
        this.textura = new Texture(Gdx.files.internal("data/regalo" + i + ".png"));
    }

    public void actualizar(Animacion animacion, boolean z) {
        if (this.seleccionado.booleanValue() || this.esperar.booleanValue()) {
            return;
        }
        if (Operaciones.puntaje > 500 && Operaciones.puntaje < 1000) {
            this.velocidad = 1.2f;
        } else if (Operaciones.puntaje > 1000 && Operaciones.puntaje < 1500) {
            this.velocidad = 1.4f;
        } else if (Operaciones.puntaje > 1500 && Operaciones.puntaje < 2000) {
            this.velocidad = 1.6f;
        } else if (Operaciones.puntaje > 2000 && Operaciones.puntaje < 2500) {
            this.velocidad = 1.8f;
        } else if (Operaciones.puntaje > 2500 && Operaciones.puntaje < 3000) {
            this.velocidad = 2.0f;
        } else if (Operaciones.puntaje > 3000 && Operaciones.puntaje < 3500) {
            this.velocidad = 2.2f;
        } else if (Operaciones.puntaje > 3500 && Operaciones.puntaje < 4000) {
            this.velocidad = 2.4f;
        } else if (Operaciones.puntaje > 4000 && Operaciones.puntaje < 4500) {
            this.velocidad = 2.6f;
        } else if (Operaciones.puntaje > 4500 && Operaciones.puntaje < 5000) {
            this.velocidad = 2.8f;
        }
        if (!z) {
            caer();
        }
        if (!Gdx.input.justTouched() || Gdx.input.getX() >= getPosicion().x + getAncho() || Gdx.input.getX() <= getPosicion().x || Gdx.graphics.getHeight() - Gdx.input.getY() >= getPosicion().y + getAlto() || Gdx.graphics.getHeight() - Gdx.input.getY() <= getPosicion().y) {
            return;
        }
        if (Operaciones.cantidadSeleccionados > 0 && Gdx.graphics.getHeight() - Gdx.input.getY() < Operaciones.posicionFranja + animacion.getAlto() && Gdx.graphics.getHeight() - Gdx.input.getY() > Operaciones.posicionFranja) {
            this.color = Color.RED;
            this.seleccionado = true;
            Operaciones.cantidadSeleccionados++;
        }
        if (Operaciones.cantidadSeleccionados == 0) {
            Operaciones.posicionFranja = getPosicion().y;
            this.color = Color.RED;
            this.seleccionado = true;
            Operaciones.cantidadSeleccionados++;
        }
        if (Operaciones.cantidadSeleccionados <= 0 || this.color == Color.RED) {
            return;
        }
        Operaciones.error = true;
        Operaciones.cantidadSeleccionados = 0;
    }

    void caer() {
        this.posicion.y -= (this.velocidad * Gdx.graphics.getDeltaTime()) * 60.0f;
    }

    public void dibujar(SpriteBatch spriteBatch) {
        Color color = spriteBatch.getColor();
        spriteBatch.setColor(this.color);
        spriteBatch.draw(this.textura, this.posicion.x, this.posicion.y, this.ancho, this.alto);
        spriteBatch.setColor(color);
    }

    public float getAlto() {
        return this.alto;
    }

    public float getAncho() {
        return this.ancho;
    }

    public Color getColor() {
        return this.color;
    }

    public Boolean getEsperar() {
        return this.esperar;
    }

    public Vector2 getPosicion() {
        return this.posicion;
    }

    public Boolean getSeleccionado() {
        return this.seleccionado;
    }

    public Texture getTextura() {
        return this.textura;
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

    public void setColor(Color color) {
        this.color = color;
    }

    public void setEsperar(Boolean bool) {
        this.esperar = bool;
    }

    public void setPosicion(int i, int i2) {
        this.posicion.x = i;
        this.posicion.y = i2;
    }

    public void setSeleccionado(Boolean bool) {
        this.seleccionado = bool;
    }

    public void setTextura(Texture texture) {
        this.textura = texture;
    }

    public void setVelocidad(float f) {
        this.velocidad = f;
    }
}
