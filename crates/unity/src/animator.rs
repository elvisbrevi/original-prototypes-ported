// Animator de Mecanim (Unity 4 y 5) con lo que usan los juegos 2D: una capa, estados con un clip
// de sprites o un blend tree 1D, parámetros float/int/bool y transiciones con condiciones, también
// desde AnyState. La duración de una transición está en tiempo normalizado del estado de origen
// (en Unity 5 puede ser fija, en segundos). Las curvas de sprite no se mezclan: durante una
// transición se ve el sprite del estado que pesa más, y en un blend tree, el del clip que pesa
// más.

use std::collections::HashMap;

use crate::datos::{Clip, Controlador, Transicion, ValorAnimado};

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
    /// AnimatorUpdateMode: 0 Normal (con Update), 1 AnimatePhysics (con cada FixedUpdate).
    pub modo: u32,
    /// El valor original de cada propiedad que animó algún clip (Write Defaults).
    pub defectos: HashMap<String, f32>,
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
            modo: 0,
            defectos: HashMap::new(),
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

    fn numero(&self, nombre: &str) -> f32 {
        match self.parametros.get(nombre) {
            Some(Valor::Numero(x)) => *x,
            _ => 0.0,
        }
    }

    /// Los pesos de los clips de un blend tree 1D con el valor actual de su parámetro.
    fn pesos(&self, estado: usize) -> Option<Vec<f32>> {
        let m = self.controlador.estados[estado].mezcla.as_ref()?;
        let x = self.numero(&m.parametro);
        let u = &m.umbrales;
        let mut w = vec![0.0; u.len()];
        if x <= u[0] {
            w[0] = 1.0;
        } else if x >= u[u.len() - 1] {
            w[u.len() - 1] = 1.0;
        } else {
            let i = (0..u.len() - 1)
                .find(|&i| x <= u[i + 1])
                .expect("entre umbrales");
            let f = (x - u[i]) / (u[i + 1] - u[i]);
            w[i] = 1.0 - f;
            w[i + 1] = f;
        }
        Some(w)
    }

    /// El largo de un estado en tiempo de clip: el de su clip, o en un blend tree, la media de
    /// los de sus clips según sus pesos.
    fn largo(&self, clips: &HashMap<String, Clip>, estado: usize) -> f32 {
        let e = &self.controlador.estados[estado];
        match (&e.mezcla, self.pesos(estado)) {
            (Some(m), Some(w)) => m
                .clips
                .iter()
                .zip(w)
                .map(|(c, w)| clips.get(c).map_or(0.0, |c| c.fin) * w)
                .sum(),
            _ => clips.get(&e.clip).map_or(0.0, |c| c.fin),
        }
    }

    fn cumple(&self, clips: &HashMap<String, Clip>, t: &Transicion) -> bool {
        if t.con_salida {
            let largo = self.largo(clips, self.estado);
            if largo <= 0.0 || self.tiempo / largo < t.salida {
                return false;
            }
        }
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
        self.largo(clips, estado) / e.velocidad.max(1e-6)
    }

    fn empezar(&mut self, clips: &HashMap<String, Clip>, t: &Transicion) {
        let duracion = if t.duracion_fija {
            t.duracion
        } else {
            t.duracion * self.duracion(clips, self.estado)
        };
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
                .find(|t| (t.destino != self.estado || t.a_si_misma) && self.cumple(clips, t))
                .or_else(|| {
                    self.controlador.estados[self.estado]
                        .transiciones
                        .iter()
                        .find(|t| self.cumple(clips, t))
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
        let (clip, bucle, t) = self.visible(clips)?;
        Some(muestrear(clip, bucle, t).to_owned())
    }

    /// El clip que se ve (el del estado que pesa más en la transición, y en un blend tree el del
    /// clip que pesa más), si hace bucle y el tiempo dentro de él.
    fn visible<'a>(&self, clips: &'a HashMap<String, Clip>) -> Option<(&'a Clip, bool, f32)> {
        let (estado, tiempo) = match &self.transicion {
            Some(tr) if tr.transcurrido / tr.duracion >= 0.5 => (tr.destino, tr.tiempo_destino),
            _ => (self.estado, self.tiempo),
        };
        let e = &self.controlador.estados[estado];
        match (&e.mezcla, self.pesos(estado)) {
            (Some(m), Some(w)) => {
                // A igual peso, el primero.
                let i = (0..w.len()).fold(0, |a, i| if w[i] > w[a] { i } else { a });
                let clip = clips.get(&m.clips[i])?;
                let largo = self.largo(clips, estado);
                let normalizado = if largo > 0.0 { tiempo / largo } else { 0.0 };
                Some((clip, e.bucle, normalizado * clip.fin))
            }
            _ => Some((clips.get(&e.clip)?, e.bucle, tiempo)),
        }
    }

    /// Las propiedades que fija el clip que se ve (ver Escena::animar_colisionador).
    pub fn valores<'a>(&self, clips: &'a HashMap<String, Clip>) -> &'a [ValorAnimado] {
        if !self.activo {
            return &[];
        }
        self.visible(clips).map_or(&[], |(c, _, _)| &c.valores)
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
            valores: Vec::new(),
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
            duracion_fija: false,
            desfase: 0.0,
            con_salida: false,
            salida: 0.0,
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
            mezcla: None,
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

    #[test]
    fn blend_tree_1d_muestra_el_clip_que_mas_pesa() {
        let (mut c, mut clips) = controlador();
        c.parametros.push(Parametro {
            nombre: "vSpeed".into(),
            tipo: "float".into(),
            defecto: Value::from(0.0),
        });
        c.estados[1].clip = String::new();
        c.estados[1].mezcla = Some(crate::datos::Mezcla1D {
            parametro: "vSpeed".into(),
            umbrales: vec![-10.0, -2.0, 2.0],
            clips: vec!["cae".into(), "arriba".into(), "sube".into()],
        });
        for n in ["cae", "arriba", "sube"] {
            clips.insert(n.into(), clip(1.0 / 60.0, &[n]));
        }
        let mut a = Animator::nuevo(true, &c);
        a.set_bool("Ground", false);
        a.set_float("vSpeed", 5.0);
        a.actualizar(&clips, 0.3);
        a.actualizar(&clips, 0.02);
        assert_eq!(a.nombre_estado(), "Jump");
        assert_eq!(a.actualizar(&clips, 0.02).as_deref(), Some("sube"));
        a.set_float("vSpeed", -5.0);
        assert_eq!(a.actualizar(&clips, 0.02).as_deref(), Some("arriba"));
        a.set_float("vSpeed", -7.0);
        assert_eq!(a.actualizar(&clips, 0.02).as_deref(), Some("cae"));
        a.set_float("vSpeed", -30.0);
        assert_eq!(a.actualizar(&clips, 0.02).as_deref(), Some("cae"));
    }
}
