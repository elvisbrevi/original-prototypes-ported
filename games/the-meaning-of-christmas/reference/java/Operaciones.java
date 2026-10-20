package com.badlogic.gdx.helloworld;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Color;
import java.util.ArrayList;
import java.util.Random;

/* JADX INFO: loaded from: classes.dex */
public class Operaciones {
    public static float altoDinero;
    public static float posicionDinero;
    public static int cantidadSeleccionados = 0;
    public static float posicionFranja = 0.0f;
    public static Boolean error = false;
    public static int puntaje = 0;

    public static float Porciento(float f, float f2) {
        return (f2 / 100.0f) * f;
    }

    public static void controlarCaida(ArrayList<Regalo> arrayList) {
        for (int i = 0; i < arrayList.size(); i++) {
            if (arrayList.get(i).getPosicion().y < posicionDinero + altoDinero) {
                if (arrayList.get(i).getColor() == Color.RED) {
                    if (posicionDinero > (-altoDinero)) {
                        posicionDinero -= (Gdx.graphics.getDeltaTime() * 10.0f) * 60.0f;
                    }
                    puntaje += 25;
                    setear(arrayList.get(i), arrayList, i);
                } else {
                    if (posicionDinero < (-Porciento(10.0f, Gdx.graphics.getHeight()))) {
                        posicionDinero += 5.0f * Gdx.graphics.getDeltaTime() * 60.0f;
                    }
                    setear(arrayList.get(i), arrayList, i);
                }
            }
        }
        for (int i2 = 0; i2 < arrayList.size(); i2++) {
            if (arrayList.get(i2).getPosicion().y < posicionDinero + altoDinero || arrayList.get(i2).getEsperar().booleanValue()) {
                setear(arrayList.get(i2), arrayList, i2);
            }
        }
    }

    static Boolean existePosicion(ArrayList<Regalo> arrayList, int i, float f, float f2, float f3) {
        boolean z = false;
        for (int i2 = 0; i2 < arrayList.size(); i2++) {
            if (arrayList.get(i2).getPosicion().x == f && i != i2) {
                z = true;
            }
        }
        return z;
    }

    static void setear(Regalo regalo, ArrayList<Regalo> arrayList, int i) {
        Random random = new Random();
        regalo.setPosicion((int) (Porciento(10.0f, Gdx.graphics.getWidth()) + Porciento(random.nextInt(8) * 15, Gdx.graphics.getWidth())), (int) (Gdx.graphics.getHeight() + (random.nextInt(5) * regalo.getAlto())));
        if (existePosicion(arrayList, i, regalo.getPosicion().x, regalo.getPosicion().y, regalo.getAlto()).booleanValue()) {
            arrayList.get(i).setEsperar(true);
        } else {
            arrayList.get(i).setEsperar(false);
        }
        arrayList.get(i).setColor(Color.WHITE);
    }
}
