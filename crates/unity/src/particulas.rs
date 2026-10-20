// ParticleSystem de Unity 4 (Shuriken), con los módulos que usan los juegos: Initial, Shape
// (esfera, semiesfera, cono y caja), Emission (ritmo y ráfagas), Size, Color y Rotation over
// Lifetime, Texture Sheet Animation y Velocity over Lifetime. La simulación local
// (moveWithTransform) guarda las partículas en el espacio del emisor; como Unity 4, el sistema no
// hace caso de la escala de su Transform. El azar es el de macroquad: el original usa su propio
// generador.

use macroquad::prelude::{Color, Mat4, Vec3, rand, vec3};

use crate::datos::{DatosParticulas, ModuloForma};

pub struct RendererParticulas {
    pub activo: bool,
    pub material: Option<String>,
    /// 0 Billboard, 1 Stretch, 2 HorizontalBillboard, 3 VerticalBillboard, 4 Mesh.
    pub modo: u32,
    pub escala_largo: f32,
    pub escala_velocidad: f32,
    /// Fracción del alto de la pantalla.
    pub tamano_maximo: f32,
    pub capa_orden: usize,
    pub orden: i32,
}

pub struct Particula {
    /// En el espacio de simulación (local del emisor o del mundo).
    pub posicion: Vec3,
    /// La velocidad inicial, en el mismo espacio.
    pub velocidad: Vec3,
    pub vida: f32,
    pub edad: f32,
    pub tamano: f32,
    pub color: Color,
    pub rotacion: f32,
    /// Azar fijo de la partícula para las curvas aleatorias: tamaño, color, cuadro, rotación y
    /// las tres componentes de la velocidad.
    pub azar: [f32; 7],
}

impl Particula {
    pub fn normalizada(&self) -> f32 {
        if self.vida > 0.0 {
            (self.edad / self.vida).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

pub struct Particulas {
    pub datos: DatosParticulas,
    pub renderer: Option<RendererParticulas>,
    pub particulas: Vec<Particula>,
    /// ParticleSystem.time.
    pub tiempo: f32,
    /// Emitiendo (isPlaying sin haber llegado al final de un sistema sin bucle).
    pub reproduciendo: bool,
    acumulado: f32,
    rafagas_hechas: Vec<bool>,
}

fn azar() -> f32 {
    rand::gen_range(0.0, 1.0)
}

/// Un punto al azar dentro de la esfera unidad.
fn dentro_de_esfera() -> Vec3 {
    loop {
        let p = vec3(azar() * 2.0 - 1.0, azar() * 2.0 - 1.0, azar() * 2.0 - 1.0);
        if p.length_squared() <= 1.0 {
            return p;
        }
    }
}

fn direccion_al_azar() -> Vec3 {
    loop {
        let p = dentro_de_esfera();
        if p.length_squared() > 1e-6 {
            return p.normalize();
        }
    }
}

/// Posición y dirección de una partícula nueva según la forma, en el espacio del emisor.
fn emitir_en(forma: Option<&ModuloForma>) -> (Vec3, Vec3) {
    let Some(f) = forma else {
        return (Vec3::ZERO, vec3(0.0, 0.0, 1.0));
    };
    let (posicion, direccion) = match f.tipo {
        // Sphere y Hemisphere: dentro; SphereShell y HemisphereShell: en la superficie.
        0..=3 => {
            let dir = direccion_al_azar();
            let r = if f.tipo % 2 == 0 { azar().cbrt() } else { 1.0 };
            let mut d = dir;
            if f.tipo >= 2 {
                d.z = d.z.abs();
            }
            (d * r * f.radio, d)
        }
        // Cone: desde un disco del radio, abriéndose en el ángulo.
        4 | 7..=9 => {
            let a = azar() * std::f32::consts::TAU;
            let r = azar().sqrt();
            let base = vec3(a.cos() * r, a.sin() * r, 0.0);
            let abre = f.angulo.to_radians().sin();
            let dir = vec3(base.x * abre, base.y * abre, 1.0).normalize();
            (base * f.radio, dir)
        }
        // Box.
        5 => (
            vec3(
                (azar() - 0.5) * f.caja[0],
                (azar() - 0.5) * f.caja[1],
                (azar() - 0.5) * f.caja[2],
            ),
            vec3(0.0, 0.0, 1.0),
        ),
        _ => (Vec3::ZERO, vec3(0.0, 0.0, 1.0)),
    };
    if f.direccion_aleatoria {
        (posicion, direccion_al_azar())
    } else {
        (posicion, direccion)
    }
}

impl Particulas {
    pub fn nuevas(datos: DatosParticulas, renderer: Option<RendererParticulas>) -> Self {
        let rafagas = datos.emision.as_ref().map_or(0, |e| e.rafagas.len());
        let reproduciendo = datos.al_despertar;
        Self {
            datos,
            renderer,
            particulas: Vec::new(),
            tiempo: 0.0,
            reproduciendo,
            acumulado: 0.0,
            rafagas_hechas: vec![false; rafagas],
        }
    }

    /// ParticleSystem.IsAlive (sin los hijos).
    pub fn vivo(&self) -> bool {
        self.reproduciendo || !self.particulas.is_empty()
    }

    fn emitir(&mut self, cantidad: u32, emisor: &Mat4) {
        let d = &self.datos;
        let t = if d.duracion > 0.0 {
            (self.tiempo / d.duracion).clamp(0.0, 1.0)
        } else {
            0.0
        };
        for _ in 0..cantidad {
            if self.particulas.len() >= d.inicial.maximo {
                return;
            }
            let (mut posicion, direccion) = emitir_en(d.forma.as_ref());
            let rapidez = d.inicial.rapidez.evaluar(t, azar());
            let mut velocidad = direccion * rapidez;
            if !d.espacio_local {
                posicion = emisor.transform_point3(posicion);
                velocidad = emisor.transform_vector3(velocidad);
            }
            self.particulas.push(Particula {
                posicion,
                velocidad,
                vida: d.inicial.vida.evaluar(t, azar()),
                edad: 0.0,
                tamano: d.inicial.tamano.evaluar(t, azar()),
                color: d.inicial.color.evaluar(t, azar()),
                rotacion: d.inicial.rotacion.evaluar(t, azar()),
                azar: [azar(), azar(), azar(), azar(), azar(), azar(), azar()],
            });
        }
    }

    /// La velocidad de una partícula en el espacio de simulación, con Velocity over Lifetime.
    pub fn velocidad(&self, p: &Particula, emisor: &Mat4) -> Vec3 {
        let mut v = p.velocidad;
        if let Some(m) = &self.datos.velocidad_en_vida {
            let t = p.normalizada();
            let extra = vec3(
                m.x.evaluar(t, p.azar[4]),
                m.y.evaluar(t, p.azar[5]),
                m.z.evaluar(t, p.azar[6]),
            );
            v += match (m.mundo, self.datos.espacio_local) {
                (false, false) => emisor.transform_vector3(extra),
                (true, true) => emisor.inverse().transform_vector3(extra),
                _ => extra,
            };
        }
        v
    }

    /// Un frame: emite según el ritmo y las ráfagas y mueve las partículas. `emisor` es la
    /// matriz del Transform sin escala.
    pub fn actualizar(&mut self, dt: f32, emisor: &Mat4) {
        let dt = dt * self.datos.velocidad_simulacion;
        // Las partículas que ya estaban envejecen y se mueven.
        let gravedad = vec3(0.0, -9.81, 0.0) * self.datos.inicial.gravedad;
        let gravedad = if self.datos.espacio_local {
            emisor.inverse().transform_vector3(gravedad)
        } else {
            gravedad
        };
        let mut i = 0;
        while i < self.particulas.len() {
            self.particulas[i].edad += dt;
            if self.particulas[i].edad >= self.particulas[i].vida {
                self.particulas.remove(i);
                continue;
            }
            self.particulas[i].velocidad += gravedad * dt;
            let v = self.velocidad(&self.particulas[i], emisor);
            self.particulas[i].posicion += v * dt;
            i += 1;
        }
        if !self.reproduciendo || dt <= 0.0 {
            return;
        }
        let duracion = self.datos.duracion.max(1e-6);
        let antes = self.tiempo;
        let despues = antes + dt;
        if let Some(e) = self.datos.emision.clone() {
            let ritmo = e.ritmo.evaluar((antes / duracion).min(1.0), azar());
            self.acumulado += ritmo * dt;
            let n = self.acumulado.floor();
            self.acumulado -= n;
            for (k, &(t, cantidad)) in e.rafagas.iter().enumerate() {
                if !self.rafagas_hechas[k] && despues >= t {
                    self.rafagas_hechas[k] = true;
                    self.emitir(cantidad, emisor);
                }
            }
            self.emitir(n as u32, emisor);
        }
        self.tiempo = despues;
        if self.tiempo >= duracion {
            if self.datos.bucle {
                self.tiempo -= duracion;
                self.rafagas_hechas.iter_mut().for_each(|r| *r = false);
            } else {
                self.tiempo = duracion;
                self.reproduciendo = false;
            }
        }
    }

    /// El tamaño, el color y el cuadro de la hoja de una partícula ahora.
    pub fn aspecto(&self, p: &Particula) -> (f32, Color, u32) {
        let t = p.normalizada();
        let d = &self.datos;
        let tamano = p.tamano
            * d.tamano_en_vida
                .as_ref()
                .map_or(1.0, |c| c.evaluar(t, p.azar[0]));
        let mut color = p.color;
        if let Some(g) = &d.color_en_vida {
            let c = g.evaluar(t, p.azar[1]);
            color = Color::new(color.r * c.r, color.g * c.g, color.b * c.b, color.a * c.a);
        }
        let cuadro = d.hoja.as_ref().map_or(0, |h| {
            let total = (h.x * h.y).max(1);
            let cuadros = if h.tipo == 0 { total } else { h.x.max(1) };
            let v = h.cuadro.evaluar(t, p.azar[2]) * h.ciclos;
            let f = ((v.fract() * cuadros as f32).floor() as u32).min(cuadros - 1);
            let f = if v >= 1.0 && v.fract() == 0.0 {
                cuadros - 1
            } else {
                f
            };
            if h.tipo == 0 { f } else { h.fila * h.x + f }
        });
        (tamano, color, cuadro)
    }
}
