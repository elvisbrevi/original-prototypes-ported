// Las clases de com.badlogic.gdx.helloworld traducidas del Java decompilado con JADX
// (reference/java): Juego (el ApplicationListener), Inicio (el título), Regalo, Operaciones (sus
// campos static viven en `Operaciones`, dentro de Juego), Cielo y Animacion. Las coordenadas son
// las de libGDX: en píxeles, con el origen abajo a la izquierda.

use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::gdx::{ALTO, ANCHO, BitmapFont, Entrada, Musica, Reloj, dibujar, dibujar_region};

/// Operaciones.Porciento(f, f2).
fn porciento(f: f32, f2: f32) -> f32 {
    (f2 / 100.0) * f
}

/// Color.RED y Color.WHITE (el código los compara por referencia: aquí basta el valor).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tinte {
    Blanco,
    Rojo,
}

impl Tinte {
    fn color(self) -> Color {
        match self {
            Tinte::Blanco => WHITE,
            Tinte::Rojo => RED,
        }
    }
}

/// Los campos static de Operaciones.
struct Operaciones {
    alto_dinero: f32,
    posicion_dinero: f32,
    cantidad_seleccionados: i32,
    posicion_franja: f32,
    error: bool,
    puntaje: i32,
}

/// Las texturas, la fuente y la música, cargadas por main.rs.
pub struct Recursos {
    pub titulo: Texture2D,
    pub imagen_titulo: Texture2D,
    pub pixel: Texture2D,
    pub dinero: Texture2D,
    pub franja: Texture2D,
    pub noche: Texture2D,
    pub luna: Texture2D,
    pub regalos: [Texture2D; 3],
    pub fuente: BitmapFont,
}

/// Animacion: una hoja de 4x1 cuadros con Animation(0.025) en bucle; el tiempo avanza en cada
/// dibujo (dibujar), por la velocidad.
struct Animacion {
    alto: f32,
    ancho: f32,
    cuadros: [Rect; 4],
    state_time: f32,
    velocidad: f32,
}

impl Animacion {
    fn cargar(textura: &Texture2D, f: f32, f2: f32) -> Self {
        let (w, h) = (
            (textura.width() as i32 / 4) as f32,
            textura.height() as i32 as f32,
        );
        Self {
            ancho: porciento(f, ANCHO),
            alto: porciento(f2, ALTO),
            cuadros: [0, 1, 2, 3].map(|i| Rect::new(i as f32 * w, 0.0, w, h)),
            state_time: 0.0,
            velocidad: 1.0,
        }
    }

    fn dibujar(&mut self, textura: &Texture2D, delta: f32, f: f32, f2: f32) {
        self.state_time += delta * self.velocidad;
        // Animation.getKeyFrame(stateTime, true).
        let i = (self.state_time / 0.025) as i32 as usize % self.cuadros.len();
        dibujar_region(textura, self.cuadros[i], f, f2, self.ancho, self.alto);
    }
}

struct Regalo {
    tipo: usize,
    posicion: Vec2,
    seleccionado: bool,
    color: Tinte,
    esperar: bool,
    ancho: f32,
    alto: f32,
    velocidad: f32,
}

impl Regalo {
    fn nuevo(tipo: usize) -> Self {
        Self {
            tipo,
            posicion: Vec2::ZERO,
            seleccionado: false,
            color: Tinte::Blanco,
            esperar: false,
            ancho: porciento(10.0, ANCHO),
            alto: porciento(10.0, ALTO),
            velocidad: 1.0,
        }
    }

    fn set_posicion(&mut self, i: i32, i2: i32) {
        self.posicion = vec2(i as f32, i2 as f32);
    }

    fn actualizar(
        &mut self,
        op: &mut Operaciones,
        animacion: &Animacion,
        z: bool,
        delta: f32,
        e: Entrada,
    ) {
        if self.seleccionado || self.esperar {
            return;
        }
        let p = op.puntaje;
        if p > 500 && p < 1000 {
            self.velocidad = 1.2;
        } else if p > 1000 && p < 1500 {
            self.velocidad = 1.4;
        } else if p > 1500 && p < 2000 {
            self.velocidad = 1.6;
        } else if p > 2000 && p < 2500 {
            self.velocidad = 1.8;
        } else if p > 2500 && p < 3000 {
            self.velocidad = 2.0;
        } else if p > 3000 && p < 3500 {
            self.velocidad = 2.2;
        } else if p > 3500 && p < 4000 {
            self.velocidad = 2.4;
        } else if p > 4000 && p < 4500 {
            self.velocidad = 2.6;
        } else if p > 4500 && p < 5000 {
            self.velocidad = 2.8;
        }
        if !z {
            self.caer(delta);
        }
        let y_toque = ALTO - e.y;
        if !e.tocado
            || e.x >= self.posicion.x + self.ancho
            || e.x <= self.posicion.x
            || y_toque >= self.posicion.y + self.alto
            || y_toque <= self.posicion.y
        {
            return;
        }
        if op.cantidad_seleccionados > 0
            && y_toque < op.posicion_franja + animacion.alto
            && y_toque > op.posicion_franja
        {
            self.color = Tinte::Rojo;
            self.seleccionado = true;
            op.cantidad_seleccionados += 1;
        }
        if op.cantidad_seleccionados == 0 {
            op.posicion_franja = self.posicion.y;
            self.color = Tinte::Rojo;
            self.seleccionado = true;
            op.cantidad_seleccionados += 1;
        }
        if op.cantidad_seleccionados <= 0 || self.color == Tinte::Rojo {
            return;
        }
        op.error = true;
        op.cantidad_seleccionados = 0;
    }

    fn caer(&mut self, delta: f32) {
        self.posicion.y -= (self.velocidad * delta) * 60.0;
    }

    fn dibujar(&self, r: &Recursos) {
        dibujar(
            &r.regalos[self.tipo - 1],
            self.posicion.x,
            self.posicion.y,
            self.ancho,
            self.alto,
            self.color.color(),
        );
    }
}

impl Operaciones {
    fn controlar_caida(&mut self, regalos: &mut [Regalo], delta: f32) {
        for i in 0..regalos.len() {
            if regalos[i].posicion.y < self.posicion_dinero + self.alto_dinero {
                if regalos[i].color == Tinte::Rojo {
                    if self.posicion_dinero > -self.alto_dinero {
                        self.posicion_dinero -= (delta * 10.0) * 60.0;
                    }
                    self.puntaje += 25;
                    Self::setear(regalos, i);
                } else {
                    if self.posicion_dinero < -porciento(10.0, ALTO) {
                        self.posicion_dinero += 5.0 * delta * 60.0;
                    }
                    Self::setear(regalos, i);
                }
            }
        }
        for i in 0..regalos.len() {
            if regalos[i].posicion.y < self.posicion_dinero + self.alto_dinero || regalos[i].esperar
            {
                Self::setear(regalos, i);
            }
        }
    }

    /// existePosicion: si otro regalo tiene la misma x.
    fn existe_posicion(regalos: &[Regalo], i: usize, f: f32) -> bool {
        regalos
            .iter()
            .enumerate()
            .any(|(i2, r)| r.posicion.x == f && i != i2)
    }

    /// setear: una x al azar entre ocho columnas (del 10 % al 115 % del ancho) y una altura al
    /// azar sobre la pantalla; si la columna está ocupada, el regalo espera.
    fn setear(regalos: &mut [Regalo], i: usize) {
        let alto = regalos[i].alto;
        let x = (porciento(10.0, ANCHO) + porciento((gen_range(0, 8) * 15) as f32, ANCHO)) as i32;
        let y = (ALTO + (gen_range(0, 5) as f32 * alto)) as i32;
        regalos[i].set_posicion(x, y);
        let px = regalos[i].posicion.x;
        regalos[i].esperar = Self::existe_posicion(regalos, i, px);
        regalos[i].color = Tinte::Blanco;
    }
}

/// Inicio: el título.
struct Inicio {
    activo: bool,
}

impl Inicio {
    fn dibujar(&mut self, op: &mut Operaciones, r: &mut Recursos, e: Entrada) {
        if e.tocado {
            op.puntaje = 0;
            op.posicion_dinero = -op.alto_dinero;
            self.activo = false;
        }
        dibujar(&r.pixel, 0.0, 0.0, ANCHO, ALTO, WHITE);
        dibujar(
            &r.titulo,
            0.0,
            ALTO - porciento(30.0, ALTO),
            porciento(100.0, ANCHO),
            porciento(30.0, ALTO),
            WHITE,
        );
        dibujar(
            &r.imagen_titulo,
            0.0,
            ALTO - porciento(110.0, ALTO),
            porciento(110.0, ANCHO),
            porciento(100.0, ALTO),
            WHITE,
        );
        r.fuente.color = GREEN;
        if op.puntaje > 0 {
            r.fuente.draw(
                &format!("puntaje: {}, haz click para empezar", op.puntaje),
                10.0,
                30.0,
            );
        } else {
            r.fuente.draw("haz click para empezar", 10.0, 30.0);
        }
    }
}

/// Cielo("luna noche"): el fondo de noche y la luna.
fn cielo(r: &Recursos) {
    dibujar(&r.noche, 0.0, 0.0, ANCHO, ALTO, WHITE);
    dibujar(
        &r.luna,
        porciento(50.0, ANCHO),
        porciento(70.0, ALTO),
        porciento(30.0, ANCHO),
        porciento(30.0, ALTO),
        WHITE,
    );
}

pub struct Juego {
    op: Operaciones,
    inicio: Inicio,
    regalos: Vec<Regalo>,
    animacion_fuego: Animacion,
    ancho_fuego: i32,
    ancho_dinero: f32,
    posicion_dinero: Vec2,
    noche: Musica,
    angel: Musica,
    reloj: Reloj,
}

impl Juego {
    /// Juego.create.
    pub fn nuevo(r: &Recursos, noche: Musica, angel: Musica) -> Self {
        let alto_dinero = ALTO;
        let mut animacion_fuego = Animacion::cargar(&r.franja, 8.0, 8.0);
        animacion_fuego.velocidad = 0.1;
        let ancho_fuego = (ANCHO / animacion_fuego.ancho) as i32;
        let mut regalos = Vec::new();
        for i in 1..=9 {
            let mut regalo = Regalo::nuevo(match i {
                1..=3 => 1,
                4..=6 => 2,
                _ => 3,
            });
            regalo.set_posicion(0, 0);
            regalos.push(regalo);
        }
        Self {
            op: Operaciones {
                alto_dinero,
                posicion_dinero: -alto_dinero,
                cantidad_seleccionados: 0,
                posicion_franja: 0.0,
                error: false,
                puntaje: 0,
            },
            inicio: Inicio { activo: true },
            regalos,
            animacion_fuego,
            ancho_fuego,
            ancho_dinero: ANCHO,
            posicion_dinero: Vec2::ZERO,
            noche,
            angel,
            reloj: Reloj::nuevo(),
        }
    }

    /// Juego.render, un frame que duró `real` segundos.
    pub fn render(&mut self, r: &mut Recursos, real: f32, e: Entrada) {
        self.reloj.frame(real);
        let delta = self.reloj.delta();
        if self.inicio.activo {
            if self.angel.is_playing() {
                self.angel.stop();
            }
            if !self.noche.is_playing() {
                self.noche.play();
            }
            self.inicio.dibujar(&mut self.op, r, e);
            return;
        }
        if self.op.posicion_dinero >= -porciento(10.0, ALTO) {
            self.inicio.activo = true;
        }
        if self.noche.is_playing() {
            self.noche.stop();
        }
        if !self.angel.is_playing() {
            self.angel.play();
        }
        for regalo in &mut self.regalos {
            regalo.actualizar(
                &mut self.op,
                &self.animacion_fuego,
                self.inicio.activo,
                delta,
                e,
            );
        }
        if self.op.error {
            for regalo in &mut self.regalos {
                regalo.color = Tinte::Blanco;
                regalo.seleccionado = false;
            }
            self.op.error = false;
        }
        self.op.controlar_caida(&mut self.regalos, delta);
        cielo(r);
        if self.op.cantidad_seleccionados > 0 {
            for i in 0..=self.ancho_fuego {
                let a = self.animacion_fuego.ancho;
                let y = self.op.posicion_franja;
                self.animacion_fuego
                    .dibujar(&r.franja, delta, i as f32 * a - a / 2.0, y);
            }
        }
        for i in 0..self.regalos.len() {
            if self.op.cantidad_seleccionados >= 3 {
                for regalo in &mut self.regalos {
                    regalo.seleccionado = false;
                }
                self.op.cantidad_seleccionados = 0;
            }
            self.regalos[i].dibujar(r);
        }
        dibujar(
            &r.dinero,
            self.posicion_dinero.x,
            self.op.posicion_dinero,
            self.ancho_dinero,
            self.op.alto_dinero,
            WHITE,
        );
        dibujar(
            &r.pixel,
            0.0,
            ALTO - porciento(8.0, ALTO),
            ANCHO,
            porciento(8.0, ALTO),
            WHITE,
        );
        r.fuente.color = BLACK;
        r.fuente.draw(
            &format!("Puntaje: {}", self.op.puntaje),
            10.0,
            ALTO - porciento(2.0, ALTO),
        );
    }
}
