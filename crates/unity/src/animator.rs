// Animator de Mecanim (Unity 4) con lo que usan los juegos 2D: una capa, estados con un clip de
// sprites cada uno, parámetros float/int/bool y transiciones con condiciones, también desde
// AnyState. En Unity 4 la duración de una transición está en tiempo normalizado del estado de
// origen. Las curvas de sprite no se mezclan: durante una transición se ve el sprite del estado
// que pesa más.

use std::collections::HashMap;

use crate::datos::{Clip, Controlador, Transicion};

#[derive(Clone, Copy, Debug)]
enum Valor {
    Numero(f32),
    Logico(bool),
}

struct TransicionActiva {
    destino: usize,
    /// Tiempo del estado de destino.
    tiempo_destino: f32,
    transcurrido: f32,
    duracion: f32,
}

pub struct Animator {
    pub activo: bool,
    controlador: Controlador,
    parametros: HashMap<String, Valor>,
    pub estado: usize,
    /// Segundos dentro del estado actual.
    pub tiempo: f32,
    transicion: Option<TransicionActiva>,
}

impl Animator {
    pub fn nuevo(activo: bool, controlador: &Controlador) -> Self {
        let parametros = controlador
            .parametros
            .iter()
            .map(|p| {
                let v = match p.tipo.as_str() {
                    "bool" | "trigger" => Valor::Logico(p.defecto.as_bool().unwrap_or(false)),
                    _ => Valor::Numero(p.defecto.as_f64().unwrap_or(0.0) as f32),
                };
                (p.nombre.clone(), v)
            })
            .collect();
        Self {
            activo,
            controlador: controlador.clone(),
            parametros,
            estado: controlador.defecto,
            tiempo: 0.0,
            transicion: None,
        }
    }

    /// Animator.SetBool.
    pub fn set_bool(&mut self, nombre: &str, v: bool) {
        if let Some(p) = self.parametros.get_mut(nombre) {
            *p = Valor::Logico(v);
        }
    }

    /// Animator.SetFloat.
    pub fn set_float(&mut self, nombre: &str, v: f32) {
        if let Some(p) = self.parametros.get_mut(nombre) {
            *p = Valor::Numero(v);
        }
    }

    /// Animator.GetBool.
    pub fn get_bool(&self, nombre: &str) -> bool {
        matches!(self.parametros.get(nombre), Some(Valor::Logico(true)))
    }

    pub fn nombre_estado(&self) -> &str {
        &self.controlador.estados[self.estado].nombre
    }

    fn cumple(&self, t: &Transicion) -> bool {
        t.condiciones.iter().all(|c| {
            let v = c.parametro.as_ref().and_then(|p| self.parametros.get(p));
            match (c.modo, v) {
                (1, Some(Valor::Logico(b))) => *b,
                (2, Some(Valor::Logico(b))) => !*b,
                (3, Some(Valor::Numero(x))) => *x > c.umbral,
                (4, Some(Valor::Numero(x))) => *x < c.umbral,
                (6, Some(Valor::Numero(x))) => *x == c.umbral,
                (7, Some(Valor::Numero(x))) => *x != c.umbral,
                _ => false,
            }
        })
    }

    /// La duración en segundos de un estado (la de su clip, a su velocidad).
    fn duracion(&self, clips: &HashMap<String, Clip>, estado: usize) -> f32 {
        let e = &self.controlador.estados[estado];
        clips.get(&e.clip).map_or(0.0, |c| c.fin) / e.velocidad.max(1e-6)
    }

    fn empezar(&mut self, clips: &HashMap<String, Clip>, t: &Transicion) {
        let largo_origen = self.duracion(clips, self.estado);
        let duracion = t.duracion * largo_origen;
        let tiempo_destino = t.desfase * self.duracion(clips, t.destino);
        if duracion <= 0.0 {
            self.estado = t.destino;
            self.tiempo = tiempo_destino;
            self.transicion = None;
        } else {
            self.transicion = Some(TransicionActiva {
                destino: t.destino,
                tiempo_destino,
                transcurrido: 0.0,
                duracion,
            });
        }
    }

    /// Un frame del Animator (Update Mode Normal: con Time.deltaTime). Devuelve el sprite que
    /// escribe en el SpriteRenderer.
    pub fn actualizar(&mut self, clips: &HashMap<String, Clip>, dt: f32) -> Option<String> {
        if !self.activo {
            return None;
        }
        // Las transiciones son atómicas: no se interrumpen.
        if self.transicion.is_none() {
            let elegida = self
                .controlador
                .desde_cualquiera
                .iter()
                .find(|t| (t.destino != self.estado || t.a_si_misma) && self.cumple(t))
                .or_else(|| {
                    self.controlador.estados[self.estado]
                        .transiciones
                        .iter()
                        .find(|t| self.cumple(t))
                })
                .cloned();
            if let Some(t) = elegida {
                self.empezar(clips, &t);
            }
        }
        let velocidad = self.controlador.estados[self.estado].velocidad;
        self.tiempo += dt * velocidad;
        if let Some(tr) = &mut self.transicion {
            let v = self.controlador.estados[tr.destino].velocidad;
            tr.tiempo_destino += dt * v;
            tr.transcurrido += dt;
            if tr.transcurrido >= tr.duracion {
                self.estado = tr.destino;
                self.tiempo = tr.tiempo_destino;
                self.transicion = None;
            }
        }
        let (estado, tiempo) = match &self.transicion {
            Some(tr) if tr.transcurrido / tr.duracion >= 0.5 => (tr.destino, tr.tiempo_destino),
            _ => (self.estado, self.tiempo),
        };
        let e = &self.controlador.estados[estado];
        let clip = clips.get(&e.clip)?;
        Some(muestrear(clip, e.bucle, tiempo).to_owned())
    }
}

/// El sprite de un clip en el tiempo `t` (la última clave que ya pasó).
pub fn muestrear(clip: &Clip, bucle: bool, t: f32) -> &str {
    let t = if clip.fin <= 0.0 {
        0.0
    } else if bucle {
        t.rem_euclid(clip.fin)
    } else {
        t.min(clip.fin)
    };
    clip.claves
        .iter()
        .rev()
        .find(|(k, _)| *k <= t + 1e-6)
        .or(clip.claves.first())
        .map_or("", |(_, s)| s.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datos::{Condicion, Estado, Parametro, Value};

    fn clip(fin: f32, sprites: &[&str]) -> Clip {
        let paso = fin / sprites.len() as f32;
        Clip {
            fps: 1.0 / paso,
            fin,
            bucle: true,
            claves: sprites
                .iter()
                .enumerate()
                .map(|(i, s)| (i as f32 * paso, s.to_string()))
                .collect(),
        }
    }

    fn transicion(destino: usize, duracion: f32, modo: u32, parametro: &str) -> Transicion {
        Transicion {
            destino,
            duracion,
            desfase: 0.0,
            atomica: true,
            a_si_misma: false,
            condiciones: vec![Condicion {
                modo,
                parametro: Some(parametro.into()),
                umbral: 0.0,
            }],
        }
    }

    /// Como el controlador del personaje: correr y, desde cualquier estado, saltar si no hay
    /// suelo.
    fn controlador() -> (Controlador, HashMap<String, Clip>) {
        let estado = |nombre: &str, transiciones| Estado {
            nombre: nombre.into(),
            clip: nombre.into(),
            velocidad: 1.0,
            bucle: true,
            transiciones,
        };
        let c = Controlador {
            parametros: vec![Parametro {
                nombre: "Ground".into(),
                tipo: "bool".into(),
                defecto: Value::Bool(true),
            }],
            estados: vec![
                estado("Run", vec![]),
                estado("Jump", vec![transicion(0, 0.25, 1, "Ground")]),
            ],
            defecto: 0,
            desde_cualquiera: vec![transicion(1, 0.5, 2, "Ground")],
        };
        let mut clips = HashMap::new();
        clips.insert("Run".into(), clip(0.4, &["r0", "r1", "r2", "r3"]));
        clips.insert("Jump".into(), clip(0.2, &["j0"]));
        (c, clips)
    }

    #[test]
    fn clip_en_bucle_y_sin_bucle() {
        let c = clip(0.4, &["a", "b", "c", "d"]);
        assert_eq!(muestrear(&c, true, 0.0), "a");
        assert_eq!(muestrear(&c, true, 0.25), "c");
        assert_eq!(muestrear(&c, true, 0.45), "a");
        assert_eq!(muestrear(&c, false, 9.0), "d");
    }

    #[test]
    fn transicion_normalizada_al_estado_de_origen() {
        let (c, clips) = controlador();
        let mut a = Animator::nuevo(true, &c);
        assert_eq!(a.actualizar(&clips, 0.1).as_deref(), Some("r1"));
        a.set_bool("Ground", false);
        // Dura 0,5 del clip de origen (0,4 s): 0,2 s. Hasta la mitad se ve el origen.
        assert_eq!(a.actualizar(&clips, 0.05).as_deref(), Some("r1"));
        assert_eq!(a.actualizar(&clips, 0.1).as_deref(), Some("j0"));
        assert_eq!(a.nombre_estado(), "Run");
        a.actualizar(&clips, 0.1);
        assert_eq!(a.nombre_estado(), "Jump");
        // AnyState no vuelve a entrar en el mismo estado.
        a.actualizar(&clips, 0.1);
        assert_eq!(a.nombre_estado(), "Jump");
        a.set_bool("Ground", true);
        // 0,25 de 0,2 s = 0,05 s.
        a.actualizar(&clips, 0.06);
        assert_eq!(a.nombre_estado(), "Run");
    }
}
