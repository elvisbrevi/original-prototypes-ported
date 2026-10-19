// Instancia de Sprite (plugins/sprite): animaciones, tamaño, ángulo, espejo, colisiones y dibujo.
// Traducido del runtime minificado (ver reference/runtime.txt de cada juego).

use std::collections::HashMap;

use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::prelude::*;

use crate::comportamientos::{Bala, Comportamiento, Movimiento, Plataforma, Solido, Torreta};
use crate::datos::{Animacion, Cuadro, Instancia, Tipo};
use crate::geometria::{Caja, Cuadrilatero, Poligono};
use crate::tiempo::Kahan;

/// Comparación de nombres sin distinguir mayúsculas (cr.equals_nocase).
pub fn igual(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

pub struct Sprite {
    pub uid: u32,
    pub tipo: usize,
    pub capa: usize,
    pub x: f64,
    pub y: f64,
    pub ancho: f64,
    pub alto: f64,
    /// Radianes, en el sentido de las agujas del reloj.
    pub angulo: f64,
    pub opacidad: f64,
    pub visible: bool,
    /// Colisiones habilitadas.
    pub colisiones: bool,
    anim: usize,
    pub cuadro: usize,
    /// Reloj de la animación (animTimer), en tiempo de juego.
    reloj: Kahan,
    inicio_cuadro: f64,
    reproduciendo: bool,
    adelante: bool,
    repeticiones: i32,
    velocidad: f64,
    /// SetAnimation pendiente (changeAnimName, changeAnimFrom).
    pendiente: String,
    desde: i32,
    /// Dentro de un disparador de animación: los cambios de animación esperan al siguiente tick.
    pub en_disparo: bool,
    /// Nombre de la última animación terminada (OnAnimationFinished).
    pub terminada: String,
    /// Si el Sprite avanza su animación en cada tick (está en la lista de objetos con tick).
    avanza: bool,
    pub comportamientos: Vec<Comportamiento>,
    pub destruido: bool,
}

impl Sprite {
    /// createInstanceFromInit + Sprite.Instance.onCreate. Propiedades: [visibilidad inicial
    /// (0 = visible), animación inicial, cuadro inicial, colisiones].
    pub fn nuevo(
        instancia: &Instancia,
        tipo: &Tipo,
        capa: usize,
        uid: u32,
        x: f64,
        y: f64,
    ) -> Self {
        let p = &instancia.propiedades;
        let nombre = p.get(1).and_then(|v| v.as_str()).unwrap_or_default();
        let anim = tipo
            .animaciones
            .iter()
            .position(|a| igual(&a.nombre, nombre))
            .unwrap_or(0);
        let a = &tipo.animaciones[anim];
        let cuadro =
            (p.get(2).and_then(|v| v.as_u64()).unwrap_or(0) as usize).min(a.cuadros.len() - 1);
        let primera = &tipo.animaciones[0];
        let comportamientos = tipo
            .comportamientos
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let props = instancia
                    .comportamientos
                    .get(i)
                    .cloned()
                    .unwrap_or_default();
                Comportamiento::nuevo(c.clase, &props, x, y, instancia.angulo)
            })
            .collect();
        Self {
            uid,
            tipo: instancia.tipo,
            capa,
            x,
            y,
            ancho: instancia.ancho,
            alto: instancia.alto,
            angulo: instancia.angulo,
            opacidad: instancia.opacidad,
            visible: p.first().and_then(|v| v.as_u64()) == Some(0),
            colisiones: p.get(3).and_then(|v| v.as_u64()) != Some(0),
            anim,
            cuadro,
            reloj: Kahan::default(),
            inicio_cuadro: 0.0,
            reproduciendo: true,
            adelante: true,
            repeticiones: 0,
            velocidad: a.velocidad,
            pendiente: String::new(),
            desde: 0,
            en_disparo: false,
            terminada: String::new(),
            avanza: !(tipo.animaciones.len() == 1 && primera.cuadros.len() == 1)
                && primera.velocidad != 0.0,
            comportamientos,
            destruido: false,
        }
    }

    pub fn animacion<'a>(&self, tipo: &'a Tipo) -> &'a Animacion {
        &tipo.animaciones[self.anim]
    }

    pub fn cuadro_actual<'a>(&self, tipo: &'a Tipo) -> &'a Cuadro {
        &tipo.animaciones[self.anim].cuadros[self.cuadro]
    }

    // --- comportamientos ---------------------------------------------------------------------

    pub fn plataforma(&self) -> Option<&Plataforma> {
        self.comportamientos.iter().find_map(|c| match c {
            Comportamiento::Plataforma(p) => Some(p),
            _ => None,
        })
    }

    pub fn plataforma_mut(&mut self) -> Option<&mut Plataforma> {
        self.comportamientos.iter_mut().find_map(|c| match c {
            Comportamiento::Plataforma(p) => Some(p),
            _ => None,
        })
    }

    pub fn bala(&self) -> Option<&Bala> {
        self.comportamientos.iter().find_map(|c| match c {
            Comportamiento::Bala(b) => Some(b),
            _ => None,
        })
    }

    pub fn bala_mut(&mut self) -> Option<&mut Bala> {
        self.comportamientos.iter_mut().find_map(|c| match c {
            Comportamiento::Bala(b) => Some(b),
            _ => None,
        })
    }

    pub fn torreta(&self) -> Option<&Torreta> {
        self.comportamientos.iter().find_map(|c| match c {
            Comportamiento::Torreta(t) => Some(t),
            _ => None,
        })
    }

    pub fn movimiento(&self) -> Option<&Movimiento> {
        self.comportamientos.iter().find_map(|c| match c {
            Comportamiento::Movimiento(m) => Some(m),
            _ => None,
        })
    }

    pub fn movimiento_mut(&mut self) -> Option<&mut Movimiento> {
        self.comportamientos.iter_mut().find_map(|c| match c {
            Comportamiento::Movimiento(m) => Some(m),
            _ => None,
        })
    }

    pub fn solido(&self) -> Option<&Solido> {
        self.comportamientos.iter().find_map(|c| match c {
            Comportamiento::Solido(s) => Some(s),
            _ => None,
        })
    }

    // --- animación ---------------------------------------------------------------------------

    /// Sprite.tick: avanza la animación. Devuelve el nombre de la animación si terminó en este
    /// tick (para el disparador OnAnimationFinished). En el original el disparador salta dentro
    /// del tick, cuando el cuadro ya quedó en el último: el tamaño es el mismo.
    pub fn tick(&mut self, tipo: &Tipo, dt: f64) -> Option<String> {
        if !self.avanza {
            return None;
        }
        self.reloj.sumar(dt);
        if !self.pendiente.is_empty() {
            self.cambiar_animacion(tipo);
        }
        let ahora = self.reloj.suma;
        let anim = &tipo.animaciones[self.anim];
        let anterior = self.cuadro;
        let duracion = anim.cuadros[self.cuadro].duracion / self.velocidad;
        let mut terminada = None;
        if self.reproduciendo && ahora >= self.inicio_cuadro + duracion {
            let n = anim.cuadros.len() as i64;
            let mut c = self.cuadro as i64 + if self.adelante { 1 } else { -1 };
            self.inicio_cuadro += duracion;
            if c >= n {
                if anim.ida_y_vuelta {
                    self.adelante = false;
                    c = n - 2;
                } else if anim.bucle {
                    c = anim.repetir_desde as i64;
                } else {
                    self.repeticiones += 1;
                    if self.repeticiones >= anim.repeticiones {
                        c = n - 1;
                        terminada = Some(self.terminar(anim));
                    } else {
                        c = anim.repetir_desde as i64;
                    }
                }
            }
            if c < 0 {
                if anim.ida_y_vuelta {
                    c = 1;
                    self.adelante = true;
                    if !anim.bucle {
                        self.repeticiones += 1;
                        if self.repeticiones >= anim.repeticiones {
                            c = 0;
                            terminada = Some(self.terminar(anim));
                        }
                    }
                } else if anim.bucle {
                    c = anim.repetir_desde as i64;
                } else {
                    self.repeticiones += 1;
                    if self.repeticiones >= anim.repeticiones {
                        c = 0;
                        terminada = Some(self.terminar(anim));
                    } else {
                        c = anim.repetir_desde as i64;
                    }
                }
            }
            self.cuadro = c.clamp(0, n - 1) as usize;
            if ahora > self.inicio_cuadro + anim.cuadros[self.cuadro].duracion / self.velocidad {
                self.inicio_cuadro = ahora;
            }
            self.cambiar_cuadro(&anim.cuadros[anterior], &anim.cuadros[self.cuadro]);
        }
        terminada
    }

    /// onAnimFinish: se detiene en el último cuadro (el disparador lo lanza quien llama).
    fn terminar(&mut self, anim: &Animacion) -> String {
        self.reproduciendo = false;
        self.terminada = anim.nombre.clone();
        anim.nombre.clone()
    }

    /// Después del disparador: animRepeats = 0.
    pub fn despues_de_terminar(&mut self) {
        self.repeticiones = 0;
    }

    /// doChangeAnim: no hace nada si ya está reproduciendo esa misma animación.
    fn cambiar_animacion(&mut self, tipo: &Tipo) {
        let anterior = &tipo.animaciones[self.anim].cuadros[self.cuadro];
        let nombre = std::mem::take(&mut self.pendiente);
        let Some(nueva) = tipo
            .animaciones
            .iter()
            .position(|a| igual(&a.nombre, &nombre))
        else {
            return;
        };
        if igual(
            &tipo.animaciones[nueva].nombre,
            &tipo.animaciones[self.anim].nombre,
        ) && self.reproduciendo
        {
            return;
        }
        self.anim = nueva;
        let a = &tipo.animaciones[nueva];
        self.velocidad = a.velocidad;
        self.cuadro = self.cuadro.min(a.cuadros.len() - 1);
        if self.desde == 1 {
            self.cuadro = 0;
        }
        self.reproduciendo = true;
        self.inicio_cuadro = self.reloj.suma;
        self.adelante = true;
        self.cambiar_cuadro(anterior, &a.cuadros[self.cuadro]);
    }

    /// OnFrameChanged interno: el tamaño sigue a la proporción del nuevo cuadro.
    fn cambiar_cuadro(&mut self, anterior: &Cuadro, nuevo: &Cuadro) {
        if anterior.recorte.w != nuevo.recorte.w {
            self.ancho *= nuevo.recorte.w as f64 / anterior.recorte.w as f64;
        }
        if anterior.recorte.h != nuevo.recorte.h {
            self.alto *= nuevo.recorte.h as f64 / anterior.recorte.h as f64;
        }
    }

    /// Set animation (desde: 0 el cuadro actual, 1 el principio). Dentro de un disparador de
    /// animación se aplica al empezar el siguiente tick.
    pub fn set_anim(&mut self, tipo: &Tipo, nombre: &str, desde: i32) {
        self.pendiente = nombre.to_owned();
        self.desde = desde;
        self.avanza = true;
        if !self.en_disparo {
            self.cambiar_animacion(tipo);
        }
    }

    /// Is animation playing: mira la animación pendiente si la hay.
    pub fn anim_es(&self, tipo: &Tipo, nombre: &str) -> bool {
        if !self.pendiente.is_empty() {
            igual(&self.pendiente, nombre)
        } else {
            igual(&tipo.animaciones[self.anim].nombre, nombre)
        }
    }

    // --- tamaño y apariencia -----------------------------------------------------------------

    /// Set scale: el tamaño del cuadro actual por la escala, conservando el espejo.
    pub fn set_escala(&mut self, tipo: &Tipo, escala: f64) {
        let c = self.cuadro_actual(tipo);
        let sx = if self.ancho < 0.0 { -1.0 } else { 1.0 };
        let sy = if self.alto < 0.0 { -1.0 } else { 1.0 };
        self.ancho = c.recorte.w as f64 * escala * sx;
        self.alto = c.recorte.h as f64 * escala * sy;
    }

    /// Set mirrored (0: espejado, 1: normal): el signo del ancho.
    pub fn set_espejo(&mut self, opcion: i32) {
        self.ancho = self.ancho.abs() * if opcion == 0 { -1.0 } else { 1.0 };
    }

    /// Set opacity, en porcentaje.
    pub fn set_opacidad(&mut self, porcentaje: f64) {
        self.opacidad = (porcentaje / 100.0).clamp(0.0, 1.0);
    }

    /// La expresión Opacity (y CompareOpacity): en porcentaje con seis decimales.
    pub fn opacidad_porcentaje(&self) -> f64 {
        crate::geometria::redondear6(100.0 * self.opacidad)
    }

    // --- colisiones --------------------------------------------------------------------------

    /// update_bbox: la caja normalizada y el cuadrilátero (sin normalizar: con ancho negativo
    /// queda espejado, que es como se dibuja).
    pub fn cajas(&self, tipo: &Tipo) -> (Caja, Cuadrilatero) {
        let o = self.cuadro_actual(tipo).origen;
        let mut caja = Caja::new(self.x, self.y, self.x + self.ancho, self.y + self.alto);
        caja.offset(-o.x * self.ancho, -o.y * self.alto);
        let cuad = if self.angulo != 0.0 {
            caja.offset(-self.x, -self.y);
            let mut q = Cuadrilatero::desde_caja_rotada(&caja, self.angulo);
            q.offset(self.x, self.y);
            caja = q.caja();
            q
        } else {
            Cuadrilatero::desde_caja(&caja)
        };
        caja.normalize();
        (caja, cuad)
    }

    /// El polígono de colisión del cuadro actual con el tamaño y el ángulo de la instancia
    /// (None si el cuadro no tiene: se usa el cuadrilátero).
    pub fn poligono(&self, tipo: &Tipo) -> Option<Poligono> {
        let c = self.cuadro_actual(tipo);
        (!c.poligono.is_empty())
            .then(|| Poligono::desde_fracciones(&c.poligono, self.ancho, self.alto, self.angulo))
    }

    /// contains_pt: el punto está en la caja, en el cuadrilátero y, si lo hay, en el polígono.
    pub fn contiene(&self, tipo: &Tipo, px: f64, py: f64) -> bool {
        let (caja, cuad) = self.cajas(tipo);
        if !(caja.contiene(px, py) && cuad.contiene(px, py)) {
            return false;
        }
        match self.poligono(tipo) {
            Some(poly) => poly.contiene(px - self.x, py - self.y),
            None => true,
        }
    }

    /// runtime.testOverlap entre dos Sprites (en capas con el mismo parallax y escala): cajas,
    /// cuadriláteros y, si alguno tiene polígono propio, polígonos.
    pub fn solapa(&self, tipo: &Tipo, otro: &Sprite, tipo_otro: &Tipo) -> bool {
        if std::ptr::eq(self, otro) || !(self.colisiones && otro.colisiones) {
            return false;
        }
        let (ca, qa) = self.cajas(tipo);
        let (cb, qb) = otro.cajas(tipo_otro);
        if !ca.se_toca(&cb) || !qa.se_corta(&qb) {
            return false;
        }
        let (pa, pb) = (self.poligono(tipo), otro.poligono(tipo_otro));
        if pa.is_none() && pb.is_none() {
            return true;
        }
        let pa = pa.unwrap_or_else(|| Poligono::desde_cuadrilatero(&qa, self.x, self.y));
        let pb = pb.unwrap_or_else(|| Poligono::desde_cuadrilatero(&qb, otro.x, otro.y));
        pa.se_corta(&pb, otro.x - self.x, otro.y - self.y)
    }

    // --- dibujo ------------------------------------------------------------------------------

    /// Sprite.drawGL: el cuadro estirado al cuadrilátero, con la opacidad.
    pub fn dibujar(&self, tipo: &Tipo, texturas: &HashMap<String, Texture2D>) {
        if !self.visible || self.ancho == 0.0 || self.alto == 0.0 {
            return;
        }
        let c = self.cuadro_actual(tipo);
        let textura = &texturas[&c.imagen];
        let (_, q) = self.cajas(tipo);
        let (tw, th) = (textura.width(), textura.height());
        let r = c.recorte;
        let (u0, v0, u1, v1) = (r.x / tw, r.y / th, (r.x + r.w) / tw, (r.y + r.h) / th);
        let color = Color::new(1.0, 1.0, 1.0, self.opacidad as f32);
        let vertice =
            |i: usize, u: f32, v: f32| Vertex::new(q.x[i] as f32, q.y[i] as f32, 0.0, u, v, color);
        draw_mesh(&Mesh {
            vertices: vec![
                vertice(0, u0, v0),
                vertice(1, u1, v0),
                vertice(2, u1, v1),
                vertice(3, u0, v1),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: Some(textura.clone()),
        });
    }
}
