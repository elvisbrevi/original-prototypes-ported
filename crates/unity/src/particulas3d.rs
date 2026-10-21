// Las partículas heredadas de Unity 3.x: EllipsoidParticleEmitter, ParticleAnimator,
// ParticleRenderer (billboards) y WorldParticleCollider. Unity las simula después de LateUpdate:
// primero emite (un emisor «One Shot» lanza toda su emisión de una vez cuando no le quedan
// partículas vivas y vuelve a hacerlo cada vez que se le acaban, mientras `emit` siga encendido),
// después las anima y las choca, y al final borra las que se quedaron sin energía; por eso un
// script ve particleCount en 0 un frame antes de que vuelvan a salir.

use macroquad::prelude::{Color, Mat4, Vec3, vec3};

use crate::datos3d::{AnimadorParticulas, Emisor};
use crate::escena3d::Id;
use crate::fisica3d::Impacto;

#[derive(Clone, Copy)]
pub struct ParticulaLegado {
    pub posicion: Vec3,
    pub velocidad: Vec3,
    pub tamano: f32,
    /// En grados.
    pub rotacion: f32,
    pub velocidad_angular: f32,
    pub energia: f32,
    pub energia_inicial: f32,
    pub color: Color,
}

pub struct SistemaLegado {
    pub emisor: Emisor,
    /// ParticleEmitter.emit.
    pub emitir: bool,
    pub animador: Option<AnimadorParticulas>,
    /// ParticleRenderer: activo, material y tamaño máximo en pantalla (fracción del alto).
    pub renderer: Option<(bool, Option<String>, f32)>,
    /// WorldParticleCollider: rebote, pérdida de energía, enviar mensaje, velocidad mínima.
    pub choque: Option<(f32, f32, bool, f32)>,
    pub particulas: Vec<ParticulaLegado>,
    acumulado: f32,
    posicion_anterior: Option<Vec3>,
}

fn azar(min: f32, max: f32) -> f32 {
    crate::azar_entre(min, max)
}

/// Un punto al azar en una esfera de radio 1.
fn en_esfera() -> Vec3 {
    loop {
        let p = vec3(azar(-1.0, 1.0), azar(-1.0, 1.0), azar(-1.0, 1.0));
        if p.length_squared() <= 1.0 {
            return p;
        }
    }
}

/// Un vector con cada componente al azar entre -e y e.
fn en_caja(e: Vec3) -> Vec3 {
    vec3(azar(-e.x, e.x), azar(-e.y, e.y), azar(-e.z, e.z))
}

impl SistemaLegado {
    pub fn nuevo(
        emisor: Emisor,
        animador: Option<AnimadorParticulas>,
        renderer: Option<(bool, Option<String>, f32)>,
        choque: Option<(f32, f32, bool, f32)>,
    ) -> Self {
        Self {
            emitir: emisor.emitir,
            emisor,
            animador,
            renderer,
            choque,
            particulas: Vec::new(),
            acumulado: 0.0,
            posicion_anterior: None,
        }
    }

    /// ParticleEmitter.particleCount.
    pub fn cantidad(&self) -> usize {
        self.particulas.len()
    }

    fn crear(&mut self, n: usize, matriz: &Mat4, velocidad_emisor: Vec3) {
        let e = &self.emisor;
        for _ in 0..n {
            // Dentro del elipsoide (sus valores son los semiejes), fuera del rango mínimo.
            let mut dir = en_esfera();
            if e.rango_minimo > 0.0 {
                let largo = dir.length().max(1e-6);
                dir = dir / largo * (e.rango_minimo + (1.0 - e.rango_minimo) * largo);
            }
            let local = dir * Vec3::from(e.elipsoide);
            let posicion = if e.mundo {
                matriz.transform_point3(local)
            } else {
                local
            };
            let tangente = if Vec3::from(e.velocidad_tangente) != Vec3::ZERO {
                let n = local.normalize_or_zero();
                Vec3::from(e.velocidad_tangente) * n.cross(Vec3::Y)
            } else {
                Vec3::ZERO
            };
            let velocidad = Vec3::from(e.velocidad_mundo)
                + matriz.transform_vector3(Vec3::from(e.velocidad_local) + tangente)
                + en_caja(Vec3::from(e.velocidad_azar) / 2.0)
                + velocidad_emisor * e.escala_velocidad_emisor;
            let energia = azar(e.energia[0], e.energia[1]);
            self.particulas.push(ParticulaLegado {
                posicion,
                velocidad,
                tamano: azar(e.tamano[0], e.tamano[1]),
                rotacion: if e.rotacion_azar {
                    azar(0.0, 360.0)
                } else {
                    0.0
                },
                velocidad_angular: e.velocidad_angular + azar(-1.0, 1.0) * e.velocidad_angular_azar,
                energia,
                energia_inicial: energia,
                color: Color::new(1.0, 1.0, 1.0, 1.0),
            });
        }
    }

    /// Un frame: emisión, animación, choques y muerte de las partículas. `rayo` busca el primer
    /// colisionador en un segmento (origen, dirección, largo). Devuelve los objetos a los que hay
    /// que mandar OnParticleCollision.
    pub fn actualizar(
        &mut self,
        dt: f32,
        matriz: &Mat4,
        rayo: &mut dyn FnMut(Vec3, Vec3, f32) -> Option<Impacto>,
    ) -> Vec<Id> {
        let posicion = matriz.transform_point3(Vec3::ZERO);
        let velocidad_emisor = match self.posicion_anterior {
            Some(p) if dt > 0.0 => (posicion - p) / dt,
            _ => Vec3::ZERO,
        };
        self.posicion_anterior = Some(posicion);
        if self.emisor.activo && self.emitir {
            if self.emisor.una_vez {
                if self.particulas.is_empty() {
                    let n = azar(self.emisor.emision[0], self.emisor.emision[1]).round() as usize;
                    self.crear(n, matriz, velocidad_emisor);
                }
            } else {
                self.acumulado += azar(self.emisor.emision[0], self.emisor.emision[1]) * dt;
                let n = self.acumulado.floor();
                self.acumulado -= n;
                self.crear(n as usize, matriz, velocidad_emisor);
            }
        }
        let mut mensajes = Vec::new();
        for p in &mut self.particulas {
            if let Some(a) = &self.animador {
                if a.animar_color {
                    let t = (1.0 - p.energia / p.energia_inicial).clamp(0.0, 1.0) * 4.0;
                    let i = (t.floor() as usize).min(3);
                    let f = t - i as f32;
                    let (c0, c1) = (a.colores[i], a.colores[i + 1]);
                    p.color = Color::new(
                        c0[0] + (c1[0] - c0[0]) * f,
                        c0[1] + (c1[1] - c0[1]) * f,
                        c0[2] + (c1[2] - c0[2]) * f,
                        c0[3] + (c1[3] - c0[3]) * f,
                    );
                }
                p.velocidad +=
                    (Vec3::from(a.fuerza) + en_caja(Vec3::from(a.fuerza_azar) / 2.0)) * dt;
                p.velocidad *= a.amortiguacion.powf(dt);
                p.tamano += p.tamano * a.crecer * dt;
            }
            p.rotacion += p.velocidad_angular * dt;
            let paso = p.velocidad * dt;
            let largo = paso.length();
            if let Some((rebote, perdida, mensaje, minima)) = self.choque
                && largo > 0.0
                && let Some(i) = rayo(p.posicion, paso / largo, largo)
            {
                p.posicion = i.punto + i.normal * 0.001;
                p.velocidad = (p.velocidad - 2.0 * p.velocidad.dot(i.normal) * i.normal) * rebote;
                p.energia -= perdida;
                if p.velocidad.length() < minima {
                    p.energia = 0.0;
                }
                if mensaje {
                    mensajes.push(i.objeto);
                }
            } else {
                p.posicion += paso;
            }
            p.energia -= dt;
        }
        self.particulas.retain(|p| p.energia > 0.0);
        mensajes
    }
}
