// La física 3D de Unity 3.x (PhysX 2.8) para lo que usan sus juegos: cuerpos esféricos
// (Rigidbody con SphereCollider) contra colisionadores estáticos (MeshCollider, BoxCollider,
// SphereCollider). PhysX es cerrado y no se puede traducir: esto es un solver propio que sigue lo
// documentado de PhysX y de Unity:
//
// - El paso: la gravedad y las fuerzas acumuladas (AddForce suma hasta el paso siguiente y se
//   borra después de él) cambian la velocidad, se buscan los contactos con la pose actual, se
//   resuelven las velocidades con impulsos secuenciales (PhysicsManager.solverIterationCount
//   iteraciones) con roce de Coulomb, se limita la velocidad angular (maxAngularVelocity) y se
//   integra la pose.
// - El «skin width» de PhysX (minPenetrationForPenalty): los cuerpos quedan apoyados con esa
//   penetración, y la que la supera se corrige moviendo el cuerpo.
// - El material por defecto (sin PhysicMaterial): roce estático y dinámico 0,6 y rebote 0.
// - Los colisionadores sin Rigidbody son estáticos: si un script los mueve, se teletransportan
//   (no empujan con velocidad, solo desplazan lo que queda dentro).
// - RigidbodyConstraints: FreezePosition anula la velocidad en ese eje.

use std::collections::HashMap;

use macroquad::prelude::{Mat4, Quat, Vec3};

use crate::datos3d::AjustesFisica;
use crate::escena3d::{Escena3d, Id};
use crate::malla::Malla;

/// Roce del material por defecto de PhysX en Unity (estático y dinámico).
pub const ROCE: f32 = 0.6;

#[derive(Clone)]
pub enum Forma3d {
    /// MeshCollider no convexo: los triángulos de colisión de la malla.
    Malla(String),
    Caja { centro: Vec3, tamano: Vec3 },
    Esfera { centro: Vec3, radio: f32 },
    /// TerrainCollider (no se simula: no hay cuerpos sobre el terreno).
    Terreno,
}

#[derive(Clone)]
pub struct Colisionador3d {
    pub activo: bool,
    pub trigger: bool,
    pub forma: Forma3d,
    /// La pose del mundo con que se calcularon los triángulos (para no recalcularlos).
    matriz: Option<Mat4>,
    triangulos: Vec<[Vec3; 3]>,
    caja: (Vec3, Vec3),
}

impl Colisionador3d {
    pub fn nuevo(activo: bool, trigger: bool, forma: Forma3d) -> Self {
        Self {
            activo,
            trigger,
            forma,
            matriz: None,
            triangulos: Vec::new(),
            caja: (Vec3::ZERO, Vec3::ZERO),
        }
    }
}

pub struct Rigidbody3d {
    pub masa: f32,
    pub arrastre: f32,
    pub arrastre_angular: f32,
    pub gravedad: bool,
    pub cinematico: bool,
    pub restricciones: u32,
    /// Rigidbody.velocity.
    pub velocidad: Vec3,
    /// Rigidbody.angularVelocity (radianes por segundo).
    pub velocidad_angular: Vec3,
    /// Las fuerzas que esperan al próximo paso (en newtons).
    pub fuerza: Vec3,
}

impl Rigidbody3d {
    pub fn nuevo(
        masa: f32,
        arrastre: f32,
        arrastre_angular: f32,
        gravedad: bool,
        cinematico: bool,
        restricciones: u32,
    ) -> Self {
        Self {
            masa,
            arrastre,
            arrastre_angular,
            gravedad,
            cinematico,
            restricciones,
            velocidad: Vec3::ZERO,
            velocidad_angular: Vec3::ZERO,
            fuerza: Vec3::ZERO,
        }
    }

    /// Rigidbody.AddForce(fuerza, ForceMode.Acceleration).
    pub fn sumar_aceleracion(&mut self, a: Vec3) {
        self.fuerza += a * self.masa;
    }

    /// Rigidbody.AddForce(fuerza, ForceMode.Impulse).
    pub fn sumar_impulso(&mut self, i: Vec3) {
        self.velocidad += i / self.masa;
    }

    fn congelar(&mut self) {
        if self.restricciones & 2 != 0 {
            self.velocidad.x = 0.0;
        }
        if self.restricciones & 4 != 0 {
            self.velocidad.y = 0.0;
        }
        if self.restricciones & 8 != 0 {
            self.velocidad.z = 0.0;
        }
        if self.restricciones & 16 != 0 {
            self.velocidad_angular.x = 0.0;
        }
        if self.restricciones & 32 != 0 {
            self.velocidad_angular.y = 0.0;
        }
        if self.restricciones & 64 != 0 {
            self.velocidad_angular.z = 0.0;
        }
    }
}

/// Un par de objetos en contacto después de un paso (para OnCollisionEnter/Stay/Exit).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Par {
    /// El objeto con el Rigidbody.
    pub cuerpo: Id,
    /// El objeto del colisionador que toca.
    pub otro: Id,
}

/// Lo que devuelve un rayo.
#[derive(Clone, Copy, Debug)]
pub struct Impacto {
    pub objeto: Id,
    pub distancia: f32,
    pub punto: Vec3,
    pub normal: Vec3,
}

struct Contacto {
    otro: Id,
    normal: Vec3,
    /// Distancia entre las superficies (negativa si se penetran).
    separacion: f32,
    normal_acumulada: f32,
    roce_acumulado: Vec3,
}

pub struct Fisica3d {
    pub ajustes: AjustesFisica,
    /// Los triángulos de colisión de cada malla, en su espacio local.
    mallas: HashMap<String, Vec<[Vec3; 3]>>,
    /// Los pares en contacto al terminar el último paso.
    pub pares: Vec<Par>,
}

fn caja_de(puntos: impl Iterator<Item = Vec3>) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for p in puntos {
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

fn se_tocan(a: (Vec3, Vec3), b: (Vec3, Vec3)) -> bool {
    a.0.x <= b.1.x
        && b.0.x <= a.1.x
        && a.0.y <= b.1.y
        && b.0.y <= a.1.y
        && a.0.z <= b.1.z
        && b.0.z <= a.1.z
}

/// El punto del triángulo más cercano a `p` (Ericson, Real-Time Collision Detection 5.1.5).
pub fn punto_cercano_triangulo(p: Vec3, [a, b, c]: [Vec3; 3]) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    a + ab * (vb * denom) + ac * (vc * denom)
}

/// Intersección de un rayo con un triángulo (Möller–Trumbore, por las dos caras).
fn rayo_triangulo(origen: Vec3, dir: Vec3, [a, b, c]: [Vec3; 3]) -> Option<f32> {
    let e1 = b - a;
    let e2 = c - a;
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let t = origen - a;
    let u = t.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = t.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let d = e2.dot(q) * inv;
    (d >= 0.0).then_some(d)
}

/// Intersección de un rayo con una caja alineada (método de los planos).
fn rayo_caja(origen: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<(f32, Vec3)> {
    let mut t0 = 0.0f32;
    let mut t1 = f32::INFINITY;
    let mut normal = Vec3::ZERO;
    for eje in 0..3 {
        let (o, d, lo, hi) = (origen[eje], dir[eje], min[eje], max[eje]);
        if d.abs() < 1e-12 {
            if o < lo || o > hi {
                return None;
            }
            continue;
        }
        let (mut a, mut b) = ((lo - o) / d, (hi - o) / d);
        let mut n = Vec3::ZERO;
        n[eje] = -d.signum();
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        if a > t0 {
            t0 = a;
            normal = n;
        }
        t1 = t1.min(b);
        if t0 > t1 {
            return None;
        }
    }
    Some((t0, normal))
}

fn rayo_esfera(origen: Vec3, dir: Vec3, centro: Vec3, radio: f32) -> Option<f32> {
    let m = origen - centro;
    let b = m.dot(dir);
    let c = m.length_squared() - radio * radio;
    if c > 0.0 && b > 0.0 {
        return None;
    }
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    Some((-b - disc.sqrt()).max(0.0))
}

impl Fisica3d {
    pub fn nueva(ajustes: &AjustesFisica, mallas: &HashMap<String, Malla>) -> Self {
        Self {
            ajustes: ajustes.clone(),
            mallas: mallas
                .iter()
                .map(|(k, m)| (k.clone(), m.triangulos_colision()))
                .collect(),
            pares: Vec::new(),
        }
    }

    /// Pone al día los triángulos y la caja de los colisionadores que se movieron.
    pub fn sincronizar(&self, escena: &mut Escena3d) {
        for id in escena.ids() {
            if escena.objeto(id).colisionadores.is_empty() {
                continue;
            }
            let m = escena.matriz_mundo(id);
            for c in &mut escena.objeto_mut(id).colisionadores {
                if c.matriz == Some(m) {
                    continue;
                }
                c.matriz = Some(m);
                match &c.forma {
                    Forma3d::Malla(nombre) => {
                        c.triangulos = self.mallas.get(nombre).map_or(Vec::new(), |t| {
                            t.iter()
                                .map(|t| t.map(|p| m.transform_point3(p)))
                                .collect()
                        });
                        c.caja = caja_de(c.triangulos.iter().flatten().copied());
                    }
                    Forma3d::Caja { centro, tamano } => {
                        let h = *tamano / 2.0;
                        c.caja = caja_de((0..8).map(|i| {
                            let s = Vec3::new(
                                if i & 1 == 0 { -1.0 } else { 1.0 },
                                if i & 2 == 0 { -1.0 } else { 1.0 },
                                if i & 4 == 0 { -1.0 } else { 1.0 },
                            );
                            m.transform_point3(*centro + h * s)
                        }));
                    }
                    Forma3d::Esfera { centro, radio } => {
                        let (centro, radio) = esfera_mundo(&m, *centro, *radio);
                        c.caja = (centro - Vec3::splat(radio), centro + Vec3::splat(radio));
                    }
                    Forma3d::Terreno => c.caja = (Vec3::ZERO, Vec3::ZERO),
                }
            }
        }
    }

    /// Los contactos de una esfera (centro y radio en el mundo) con los colisionadores sólidos de
    /// los otros objetos, a menos de `margen` de separación.
    fn contactos_esfera(
        escena: &Escena3d,
        propio: Id,
        centro: Vec3,
        radio: f32,
        margen: f32,
    ) -> Vec<Contacto> {
        let mut contactos: Vec<Contacto> = Vec::new();
        let alcance = radio + margen;
        let caja = (centro - Vec3::splat(alcance), centro + Vec3::splat(alcance));
        let mut agregar = |otro: Id, punto: Vec3| {
            let d = centro - punto;
            let dist = d.length();
            if dist >= alcance || dist < 1e-6 {
                return;
            }
            let normal = d / dist;
            // Los triángulos vecinos de una misma superficie dan el mismo contacto.
            if let Some(c) = contactos
                .iter_mut()
                .find(|c| c.otro == otro && c.normal.dot(normal) > 0.999)
            {
                c.separacion = c.separacion.min(dist - radio);
                return;
            }
            contactos.push(Contacto {
                otro,
                normal,
                separacion: dist - radio,
                normal_acumulada: 0.0,
                roce_acumulado: Vec3::ZERO,
            });
        };
        for id in escena.ids() {
            if id == propio || !escena.activo_en_jerarquia(id) {
                continue;
            }
            let o = escena.objeto(id);
            if o.cuerpo.is_some() {
                continue;
            }
            for c in &o.colisionadores {
                if !c.activo || c.trigger || !se_tocan(c.caja, caja) {
                    continue;
                }
                let Some(m) = c.matriz else {
                    continue;
                };
                match &c.forma {
                    Forma3d::Malla(_) => {
                        for t in &c.triangulos {
                            let tc = caja_de(t.iter().copied());
                            if se_tocan(tc, caja) {
                                agregar(id, punto_cercano_triangulo(centro, *t));
                            }
                        }
                    }
                    Forma3d::Caja { centro: cc, tamano } => {
                        let inv = m.inverse();
                        let local = inv.transform_point3(centro) - *cc;
                        let h = *tamano / 2.0;
                        let cerca = local.clamp(-h, h);
                        if cerca != local {
                            agregar(id, m.transform_point3(cerca + *cc));
                        } else {
                            // El centro dentro de la caja: la cara más cercana.
                            let d = h - local.abs();
                            let eje = if d.x < d.y && d.x < d.z {
                                0
                            } else if d.y < d.z {
                                1
                            } else {
                                2
                            };
                            let mut p = local;
                            p[eje] = h[eje] * local[eje].signum();
                            agregar(id, m.transform_point3(p + *cc));
                        }
                    }
                    Forma3d::Esfera { centro: cc, radio: r } => {
                        let (c2, r2) = esfera_mundo(&m, *cc, *r);
                        let d = centro - c2;
                        let dist = d.length();
                        if dist > 1e-6 {
                            agregar(id, c2 + d / dist * r2);
                        }
                    }
                    Forma3d::Terreno => {}
                }
            }
        }
        contactos
    }

    /// Un paso de la simulación de `dt` segundos. Deja en `pares` los contactos del paso.
    pub fn paso(&mut self, escena: &mut Escena3d, dt: f32) {
        self.sincronizar(escena);
        let gravedad = Vec3::from(self.ajustes.gravedad);
        let skin = self.ajustes.penetracion_minima;
        let mut pares = Vec::new();
        for id in escena.ids() {
            if !escena.activo_en_jerarquia(id) {
                continue;
            }
            let Some((centro_local, radio_local)) =
                escena.objeto(id).cuerpo.as_ref().and_then(|_| {
                    escena.objeto(id).colisionadores.iter().find_map(|c| match c.forma {
                        Forma3d::Esfera { centro, radio } if c.activo && !c.trigger => {
                            Some((centro, radio))
                        }
                        _ => None,
                    })
                })
            else {
                // Un Rigidbody sin colisionador (los «OuterCore» de las llamas, sin gravedad)
                // solo integra su velocidad.
                if let Some(c) = escena.objeto_mut(id).cuerpo.as_mut()
                    && !c.cinematico
                {
                    let a = if c.gravedad { gravedad } else { Vec3::ZERO } + c.fuerza / c.masa;
                    c.velocidad += a * dt;
                    c.fuerza = Vec3::ZERO;
                    let v = c.velocidad * dt;
                    escena.objeto_mut(id).local.posicion += v;
                }
                continue;
            };
            let m = escena.matriz_mundo(id);
            let (centro, radio) = esfera_mundo(&m, centro_local, radio_local);
            let masa;
            {
                let c = escena.objeto_mut(id).cuerpo.as_mut().expect("cuerpo");
                if c.cinematico {
                    continue;
                }
                masa = c.masa;
                let a = if c.gravedad { gravedad } else { Vec3::ZERO } + c.fuerza / c.masa;
                c.velocidad += a * dt;
                c.fuerza = Vec3::ZERO;
                c.velocidad *= (1.0 - dt * c.arrastre).max(0.0);
                c.velocidad_angular *= (1.0 - dt * c.arrastre_angular).max(0.0);
            }
            // Los contactos con la pose actual, más los que quedan a menos de lo que se mueve en
            // el paso (contactos especulativos, para no atravesar los planos).
            let v = escena.objeto(id).cuerpo.as_ref().expect("cuerpo").velocidad;
            let margen = skin + v.length() * dt;
            let mut contactos = Self::contactos_esfera(escena, id, centro, radio, margen);
            let c = escena.objeto_mut(id).cuerpo.as_mut().expect("cuerpo");
            // Esfera maciza: I = 2/5 m r².
            let inercia_inv = 1.0 / (0.4 * masa * radio * radio);
            for _ in 0..self.ajustes.iteraciones {
                for k in &mut contactos {
                    let r = -k.normal * radio;
                    let vc = c.velocidad + c.velocidad_angular.cross(r);
                    // Normal: no acercarse más que hasta quedar con `skin` de penetración.
                    let error = k.separacion + skin;
                    let objetivo = if error > 0.0 { -error / dt } else { 0.0 };
                    let vn = vc.dot(k.normal);
                    let lambda = (objetivo - vn) * masa;
                    let nueva = (k.normal_acumulada + lambda).max(0.0);
                    let dl = nueva - k.normal_acumulada;
                    k.normal_acumulada = nueva;
                    c.velocidad += k.normal * dl / masa;
                    // Roce en el plano tangente, con el límite de Coulomb sobre lo acumulado.
                    let vc = c.velocidad + c.velocidad_angular.cross(r);
                    let vt = vc - k.normal * vc.dot(k.normal);
                    let largo = vt.length();
                    if largo > 1e-9 {
                        let t = vt / largo;
                        let rt = r.cross(t);
                        let masa_t = 1.0 / (1.0 / masa + rt.dot(rt) * inercia_inv);
                        let mut nuevo = k.roce_acumulado - t * largo * masa_t;
                        let maximo = ROCE * k.normal_acumulada;
                        if nuevo.length() > maximo {
                            nuevo = nuevo.normalize_or_zero() * maximo;
                        }
                        let di = nuevo - k.roce_acumulado;
                        k.roce_acumulado = nuevo;
                        c.velocidad += di / masa;
                        c.velocidad_angular += r.cross(di) * inercia_inv;
                    }
                }
            }
            c.congelar();
            let maximo = self.ajustes.velocidad_angular_maxima;
            if c.velocidad_angular.length() > maximo {
                c.velocidad_angular = c.velocidad_angular.normalize() * maximo;
            }
            let (v, w) = (c.velocidad, c.velocidad_angular);
            let mut nueva = centro + v * dt;
            // La penetración que pasa del skin se corrige moviendo el cuerpo.
            let despues = Self::contactos_esfera(escena, id, nueva, radio, skin);
            for k in &despues {
                let exceso = -(k.separacion + skin);
                if exceso > 0.0 {
                    nueva += k.normal * exceso;
                }
            }
            if escena.objeto(id).cuerpo.as_ref().expect("cuerpo").restricciones & 8 != 0 {
                nueva.z = centro.z;
            }
            let rot = escena.rotacion(id);
            let angulo = w.length() * dt;
            let rot = if angulo > 0.0 {
                (Quat::from_axis_angle(w / w.length(), angulo) * rot).normalize()
            } else {
                rot
            };
            escena.fijar_posicion(id, nueva - (escena.rotacion(id) * centro_local));
            escena.fijar_rotacion(id, rot);
            let mut otros: Vec<Id> = despues
                .iter()
                .chain(contactos.iter().filter(|k| k.normal_acumulada > 0.0))
                .map(|k| k.otro)
                .collect();
            otros.sort_unstable();
            otros.dedup();
            pares.extend(otros.into_iter().map(|otro| Par { cuerpo: id, otro }));
        }
        self.pares = pares;
    }

    /// Physics.Raycast: el primer colisionador (no trigger si así lo dicen los ajustes) que
    /// cruza el rayo hasta `distancia`.
    pub fn rayo(
        &self,
        escena: &Escena3d,
        origen: Vec3,
        dir: Vec3,
        distancia: f32,
        ignorar: Option<Id>,
    ) -> Option<Impacto> {
        let dir = dir.normalize();
        let fin = origen + dir * distancia;
        let caja_rayo = caja_de([origen, fin].into_iter());
        let mut mejor: Option<Impacto> = None;
        for id in escena.ids() {
            if Some(id) == ignorar || !escena.activo_en_jerarquia(id) {
                continue;
            }
            for c in &escena.objeto(id).colisionadores {
                if !c.activo
                    || (c.trigger && !self.ajustes.rayos_tocan_triggers)
                    || !se_tocan(c.caja, caja_rayo)
                {
                    continue;
                }
                let Some(m) = c.matriz else {
                    continue;
                };
                let impacto = match &c.forma {
                    Forma3d::Malla(_) => c
                        .triangulos
                        .iter()
                        .filter_map(|t| rayo_triangulo(origen, dir, *t).map(|d| (d, *t)))
                        .min_by(|a, b| a.0.total_cmp(&b.0))
                        .map(|(d, [a, b, cc])| {
                            let mut n = (b - a).cross(cc - a).normalize_or_zero();
                            if n.dot(dir) > 0.0 {
                                n = -n;
                            }
                            (d, n)
                        }),
                    Forma3d::Caja { centro, tamano } => {
                        let inv = m.inverse();
                        let o = inv.transform_point3(origen);
                        let d = inv.transform_vector3(dir);
                        let h = *tamano / 2.0;
                        rayo_caja(o, d, *centro - h, *centro + h).map(|(t, n)| {
                            let p = m.transform_point3(o + d * t);
                            let normal = m
                                .inverse()
                                .transpose()
                                .transform_vector3(n)
                                .normalize_or_zero();
                            ((p - origen).length(), normal)
                        })
                    }
                    Forma3d::Esfera { centro, radio } => {
                        let (cc, r) = esfera_mundo(&m, *centro, *radio);
                        rayo_esfera(origen, dir, cc, r)
                            .map(|t| (t, (origen + dir * t - cc).normalize_or_zero()))
                    }
                    Forma3d::Terreno => None,
                };
                if let Some((d, n)) = impacto
                    && d <= distancia
                    && mejor.is_none_or(|b| d < b.distancia)
                {
                    mejor = Some(Impacto {
                        objeto: id,
                        distancia: d,
                        punto: origen + dir * d,
                        normal: n,
                    });
                }
            }
        }
        mejor
    }
}

/// El centro y el radio en el mundo de un SphereCollider: Unity escala el radio por el mayor de
/// los tres ejes de la escala.
pub fn esfera_mundo(m: &Mat4, centro: Vec3, radio: f32) -> (Vec3, f32) {
    let (escala, _, _) = m.to_scale_rotation_translation();
    let e = escala.abs();
    (m.transform_point3(centro), radio * e.x.max(e.y).max(e.z))
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec3;

    #[test]
    fn punto_cercano_en_cara_arista_y_vertice() {
        let t = [Vec3::ZERO, vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)];
        let p = punto_cercano_triangulo(vec3(0.2, 1.0, 0.2), t);
        assert!((p - vec3(0.2, 0.0, 0.2)).length() < 1e-6);
        assert_eq!(
            punto_cercano_triangulo(vec3(-1.0, 1.0, -1.0), t),
            Vec3::ZERO
        );
        let p = punto_cercano_triangulo(vec3(1.0, 0.0, 1.0), t);
        assert!((p - vec3(0.5, 0.0, 0.5)).length() < 1e-6);
    }

    #[test]
    fn rayo_contra_triangulo_y_caja() {
        let t = [
            vec3(-1.0, -1.0, 0.0),
            vec3(1.0, -1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ];
        let d = rayo_triangulo(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0), t);
        assert_eq!(d, Some(5.0));
        let (d, n) = rayo_caja(
            vec3(0.0, 0.0, -5.0),
            vec3(0.0, 0.0, 1.0),
            Vec3::splat(-1.0),
            Vec3::splat(1.0),
        )
        .expect("toca");
        assert_eq!(d, 4.0);
        assert_eq!(n, vec3(0.0, 0.0, -1.0));
    }
}
