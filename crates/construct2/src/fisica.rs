// Comportamiento Physics con el motor asm.js (Box2D 2.3.1, crates/box2d): el mundo compartido,
// la creación de los cuerpos (con la descomposición de polígonos cóncavos del runtime), el tick
// que sincroniza instancia y cuerpo, las articulaciones y la gravedad. La escala es la del
// original: 1 píxel = 0,02 m.

use std::collections::VecDeque;

use box2d::shapes::{CircleShape, PolygonShape};
use box2d::world::{BodyDef, BodyType, FixtureDef, World};
use box2d::{Shape, vec2 as b2};
use macroquad::math::{DVec2, dvec2};

use crate::comportamientos::{Comportamiento, Fisica, RegistroArticulacion};
use crate::geometria::{Poligono, a_radianes};
use crate::mundo::{Inst, Mundo};

const ESCALA: f64 = 0.02;

/// El mundo de Box2D del comportamiento: se crea una vez y dura toda la partida (sobrevive a
/// los cambios de layout, como el tipo de comportamiento del original).
pub struct MundoFisico {
    pub world: World,
    /// La gravedad actual (SetWorldGravity no hace nada si es la misma).
    pub gravedad: f64,
    /// Último tick en que se avanzó el mundo: se avanza una vez por tick.
    pub ultimo_tick: i64,
    /// Stepping mode: fijo (1/60 por tick) o según el dt (como mucho 1/30).
    pub paso_fijo: bool,
    pub iteraciones_velocidad: i32,
    pub iteraciones_posicion: i32,
}

impl Default for MundoFisico {
    fn default() -> Self {
        Self {
            world: World::new(b2(0.0, 10.0)),
            gravedad: 10.0,
            ultimo_tick: -1,
            paso_fijo: true,
            iteraciones_velocidad: 8,
            iteraciones_posicion: 3,
        }
    }
}

/// isOnSegment del separador de polígonos (con 0,1 px de tolerancia).
fn en_segmento(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    let dentro = (x1 + 0.1 >= px && px >= x2 - 0.1 || x1 - 0.1 <= px && px <= x2 + 0.1)
        && (y1 + 0.1 >= py && py >= y2 - 0.1 || y1 - 0.1 <= py && py <= y2 + 0.1);
    if !dentro {
        return false;
    }
    if 0.1 < (x2 - x1).abs() {
        0.1 > ((y2 - y1) / (x2 - x1) * (px - x1) + y1 - py).abs()
    } else {
        0.1 > (px - x1).abs()
    }
}

/// calcShapes: parte un polígono cóncavo en convexos cortando en cada vértice reflejo con un
/// rayo hasta el lado más cercano. Devuelve vacío si no lo consigue.
fn separar(entrada: Vec<DVec2>) -> Vec<Vec<DVec2>> {
    let mut pendientes: VecDeque<Vec<DVec2>> = VecDeque::from([entrada]);
    let mut resultado = Vec::new();
    while let Some(p) = pendientes.front().cloned() {
        let x = p.len();
        let mut convexo = true;
        for n in 0..x {
            let da = n;
            let e = if n + 1 < x { n + 1 } else { n + 1 - x };
            let g = if n + 2 < x { n + 2 } else { n + 2 - x };
            let (k, w, o) = (p[da], p[e], p[g]);
            let det = k.x * w.y + w.x * o.y + o.x * k.y - k.y * w.x - w.y * o.x - o.y * k.x;
            if det >= 0.0 {
                continue;
            }
            convexo = false;
            let mut mejor = 1e9;
            let (mut lado_a, mut lado_b, mut corte) = (0, 0, DVec2::ZERO);
            for j in 0..x {
                if j == da || j == e {
                    continue;
                }
                let j2 = if j + 1 < x { j + 1 } else { 0 };
                let (a, d) = (p[j], p[j2]);
                let (rx, ry) = (w.x - k.x, w.y - k.y);
                let (sx, sy) = (d.x - a.x, d.y - a.y);
                let t = (sx * (a.y - k.y) - sy * (a.x - k.x)) / (ry * sx - rx * sy);
                let (hx, hy) = (k.x + t * rx, k.y + t * ry);
                if en_segmento(w.x, w.y, k.x, k.y, hx, hy)
                    && en_segmento(hx, hy, a.x, a.y, d.x, d.y)
                {
                    let (dx, dy) = (w.x - hx, w.y - hy);
                    let dist = dx * dx + dy * dy;
                    if dist < mejor {
                        lado_a = j;
                        lado_b = j2;
                        corte = dvec2(hx, hy);
                        mejor = dist;
                    }
                }
            }
            if mejor == 1e9 {
                return Vec::new();
            }
            let (a, d) = (p[lado_a], p[lado_b]);
            let mut v1 = Vec::new();
            let mut v2 = Vec::new();
            if !((d.x - corte.x).abs() < 0.1 && (d.y - corte.y).abs() < 0.1) {
                v1.push(corte);
            }
            if !((a.x - corte.x).abs() < 0.1 && (a.y - corte.y).abs() < 0.1) {
                v2.push(corte);
            }
            let mut h: i64 = -1;
            let mut s = da as i64;
            loop {
                if s != lado_b as i64 {
                    v1.push(p[s as usize]);
                } else {
                    if h < 0 || h >= x as i64 {
                        return Vec::new();
                    }
                    let ph = p[h as usize];
                    if !en_segmento(d.x, d.y, ph.x, ph.y, k.x, k.y) {
                        v1.push(p[s as usize]);
                    }
                    break;
                }
                h = s;
                s = if s - 1 < 0 { x as i64 - 1 } else { s - 1 };
            }
            v1.reverse();
            h = -1;
            s = e as i64;
            loop {
                if s != lado_a as i64 {
                    v2.push(p[s as usize]);
                } else {
                    if h < 0 || h >= x as i64 {
                        return Vec::new();
                    }
                    let ph = p[h as usize];
                    if !en_segmento(a.x, a.y, ph.x, ph.y, w.x, w.y) {
                        v2.push(p[s as usize]);
                    }
                    break;
                }
                h = s;
                s = if s + 1 > x as i64 - 1 { 0 } else { s + 1 };
            }
            pendientes.push_back(v1);
            pendientes.push_back(v2);
            pendientes.pop_front();
            break;
        }
        if convexo {
            resultado.push(pendientes.pop_front().expect("polígono"));
        }
    }
    resultado
}

/// Las piezas convexas que no son despreciables (al menos una milésima del área de la caja).
fn descomponer(puntos: Vec<DVec2>, area: f64) -> Vec<Vec<DVec2>> {
    separar(puntos)
        .into_iter()
        .filter(|parte| {
            let mut suma = 0.0;
            for i in 0..parte.len() {
                let (d, a) = (parte[i], parte[(i + 1) % parte.len()]);
                suma += d.x * a.y - d.y * a.x;
            }
            (suma / 2.0).abs() >= 0.001 * area
        })
        .collect()
}

fn fisica(m: &Mundo, i: usize) -> Option<&Fisica> {
    match &m.instancias[i] {
        Inst::Sprite(s) => s.comportamientos.iter().find_map(|c| match c {
            Comportamiento::Fisica(f) => Some(f),
            _ => None,
        }),
        _ => None,
    }
}

fn fisica_mut(m: &mut Mundo, i: usize) -> Option<&mut Fisica> {
    match &mut m.instancias[i] {
        Inst::Sprite(s) => s.comportamientos.iter_mut().find_map(|c| match c {
            Comportamiento::Fisica(f) => Some(f),
            _ => None,
        }),
        _ => None,
    }
}

impl Mundo {
    fn mundo_fisico(&mut self) -> &mut MundoFisico {
        self.fisica.get_or_insert_with(MundoFisico::default)
    }

    /// El cuerpo de Box2D de una instancia, si tiene.
    pub fn cuerpo(&self, i: usize) -> Option<box2d::BodyId> {
        fisica(self, i).and_then(|f| f.cuerpo)
    }

    /// postCreate y la creación (o recreación) del cuerpo: createBody.
    pub(crate) fn crear_cuerpo(&mut self, i: usize) {
        let Some(f) = fisica(self, i).cloned() else {
            return;
        };
        if !f.habilitada {
            return;
        }
        let mut recrea = None;
        if let Some(cuerpo) = f.cuerpo {
            let b = self.fisica.as_ref().expect("mundo").world.body(cuerpo);
            recrea = Some((b.linear_velocity, b.angular_velocity));
            for uid in f.unidas.clone() {
                if let Some(j) = self.por_uid(uid) {
                    self.destruir_articulaciones(j);
                }
            }
            self.destruir_cuerpo(i);
        }
        let s = self.sprite(i);
        let tipo = &self.proyecto.tipos[s.tipo];
        let (cx, cy) = s.centro(tipo);
        let def = BodyDef {
            body_type: if f.inmovil {
                BodyType::Static
            } else {
                BodyType::Dynamic
            },
            position: b2((ESCALA * cx) as f32, (ESCALA * cy) as f32),
            angle: s.angulo as f32,
            fixed_rotation: f.sin_rotacion,
            linear_damping: f.amortiguacion_lineal as f32,
            angular_damping: f.amortiguacion_angular as f32,
            bullet: f.bala,
            user_data: s.uid as usize,
            ..Default::default()
        };
        let cuadro = s.cuadro_actual(tipo);
        let tiene_poligono = !cuadro.poligono.is_empty();
        let mascara = if !tiene_poligono && f.mascara == 0 {
            1
        } else {
            f.mascara
        };
        let ancho = s.ancho.abs().max(1.0);
        let alto = s.alto.abs().max(1.0);
        let espejo_x = s.ancho < 0.0;
        let espejo_y = s.alto < 0.0;
        let caja = |hx: f64, hy: f64| {
            let mut p = PolygonShape::default();
            p.set_as_box((0.01 * hx) as f32, (0.01 * hy) as f32);
            Shape::Polygon(p)
        };
        let mut formas = Vec::new();
        if mascara == 0 {
            // El polígono con el ángulo a 0, relativo al centro de la caja (que con ángulo 0
            // está a (0,5 − origen) · tamaño del origen).
            let o = cuadro.origen;
            let recto = dvec2((0.5 - o.x) * s.ancho, (0.5 - o.y) * s.alto);
            let poly = Poligono::desde_fracciones(
                &cuadro.poligono,
                if espejo_x { -ancho } else { ancho },
                if espejo_y { -alto } else { alto },
                0.0,
            );
            let mut puntos: Vec<DVec2> = poly.puntos.iter().map(|p| *p - recto).collect();
            if espejo_x != espejo_y {
                puntos.reverse();
            }
            let partes = descomponer(puntos, ancho * alto);
            if partes.is_empty() {
                formas.push(caja(ancho, alto));
            } else {
                for parte in partes {
                    let vs: Vec<box2d::Vec2> = parte
                        .iter()
                        .map(|p| b2((ESCALA * p.x) as f32, (ESCALA * p.y) as f32))
                        .collect();
                    let mut p = PolygonShape::default();
                    p.set(&vs);
                    formas.push(Shape::Polygon(p));
                }
            }
        } else if mascara == 1 {
            formas.push(caja(ancho, alto));
        } else {
            formas.push(Shape::Circle(CircleShape::new(
                (0.01 * ancho.min(alto)) as f32,
            )));
        }
        let (s_ancho, s_alto) = (s.ancho, s.alto);
        let mf = self.mundo_fisico();
        let cuerpo = mf.world.create_body(&def);
        for forma in formas {
            mf.world.create_fixture(
                cuerpo,
                &FixtureDef {
                    shape: forma,
                    friction: f.friccion as f32,
                    restitution: f.elasticidad as f32,
                    density: f.densidad as f32,
                    is_sensor: false,
                },
            );
        }
        let f = fisica_mut(self, i).expect("física");
        f.cuerpo = Some(cuerpo);
        f.ultimo_ancho = s_ancho;
        f.ultimo_alto = s_alto;
        if let Some((v, w)) = recrea {
            let world = &mut self.fisica.as_mut().expect("mundo").world;
            world.body_mut(cuerpo).set_linear_velocity(v);
            world.body_mut(cuerpo).set_angular_velocity(w);
            self.rehacer_articulaciones(i);
            for uid in fisica(self, i)
                .map(|f| f.unidas.clone())
                .unwrap_or_default()
            {
                if let Some(j) = self.por_uid(uid) {
                    self.rehacer_articulaciones(j);
                }
            }
        }
    }

    /// destroyMyJoints.
    fn destruir_articulaciones(&mut self, i: usize) {
        let Some(f) = fisica_mut(self, i) else {
            return;
        };
        let juntas = std::mem::take(&mut f.articulaciones);
        if let Some(mf) = self.fisica.as_mut() {
            for j in juntas {
                mf.world.destroy_joint(j);
            }
        }
    }

    /// destroyBody (con sus articulaciones).
    fn destruir_cuerpo(&mut self, i: usize) {
        self.destruir_articulaciones(i);
        let Some(f) = fisica_mut(self, i) else {
            return;
        };
        if let Some(cuerpo) = f.cuerpo.take()
            && let Some(mf) = self.fisica.as_mut()
        {
            mf.world.destroy_body(cuerpo);
        }
    }

    /// recreateMyJoints.
    fn rehacer_articulaciones(&mut self, i: usize) {
        let registros = fisica(self, i)
            .map(|f| f.registros.clone())
            .unwrap_or_default();
        for r in registros {
            match r {
                RegistroArticulacion::Revolucion { punto, objetivo } => {
                    self.articulacion(i, punto, objetivo, None)
                }
                RegistroArticulacion::RevolucionLimitada {
                    punto,
                    objetivo,
                    inferior,
                    superior,
                } => self.articulacion(i, punto, objetivo, Some((inferior, superior))),
            }
        }
    }

    /// doCreateRevoluteJoint / doCreateLimitedRevoluteJoint: el ancla es el punto de imagen de
    /// esta instancia.
    fn articulacion(&mut self, i: usize, punto: i32, objetivo: u32, limites: Option<(f64, f64)>) {
        if !fisica(self, i).is_some_and(|f| f.habilitada) {
            return;
        }
        let Some(j) = self.por_uid(objetivo) else {
            return;
        };
        if j == i {
            return;
        }
        let (Some(cuerpo_a), Some(cuerpo_b)) = (self.cuerpo(i), self.cuerpo(j)) else {
            return;
        };
        let uid = self.uid(i);
        if let Some(fj) = fisica_mut(self, j)
            && !fj.unidas.contains(&uid)
        {
            fj.unidas.push(uid);
        }
        let s = self.sprite(i);
        let (ax, ay) = s.punto_imagen(&self.proyecto.tipos[s.tipo], punto);
        let mf = self.mundo_fisico();
        let mut def = mf.world.revolute_joint_def(
            cuerpo_a,
            cuerpo_b,
            b2((ESCALA * ax) as f32, (ESCALA * ay) as f32),
        );
        if let Some((inferior, superior)) = limites {
            def.enable_limit = true;
            def.lower_angle = a_radianes(inferior) as f32;
            def.upper_angle = a_radianes(superior) as f32;
        }
        let junta = mf.world.create_joint(&def);
        fisica_mut(self, i)
            .expect("física")
            .articulaciones
            .push(junta);
    }

    /// Physics.CreateRevoluteJoint(punto de imagen, otro objeto).
    pub fn crear_articulacion(
        &mut self,
        i: usize,
        punto: i32,
        objetivo: usize,
        limites: Option<(f64, f64)>,
    ) {
        if !fisica(self, i).is_some_and(|f| f.habilitada) {
            return;
        }
        if objetivo == i || self.cuerpo(objetivo).is_none() {
            return;
        }
        let uid = self.uid(objetivo);
        let registro = match limites {
            None => RegistroArticulacion::Revolucion {
                punto,
                objetivo: uid,
            },
            Some((inferior, superior)) => RegistroArticulacion::RevolucionLimitada {
                punto,
                objetivo: uid,
                inferior,
                superior,
            },
        };
        fisica_mut(self, i)
            .expect("física")
            .registros
            .push(registro);
        self.articulacion(i, punto, uid, limites);
    }

    /// Physics.SetAngularVelocity, en grados por segundo.
    pub fn velocidad_angular(&mut self, i: usize, grados: f64) {
        if !fisica(self, i).is_some_and(|f| f.habilitada) {
            return;
        }
        if let (Some(c), Some(mf)) = (self.cuerpo(i), self.fisica.as_mut()) {
            let b = mf.world.body_mut(c);
            b.set_angular_velocity(a_radianes(grados) as f32);
            b.set_awake(true);
        }
    }

    /// Physics.SetWorldGravity: despierta a todos los cuerpos si cambia.
    pub fn gravedad_mundo(&mut self, g: f64) {
        if self.mundo_fisico().gravedad == g {
            return;
        }
        self.mundo_fisico().world.set_gravity(b2(0.0, g as f32));
        self.mundo_fisico().gravedad = g;
        for i in 0..self.instancias.len() {
            if let (Some(c), Some(mf)) = (self.cuerpo(i), self.fisica.as_mut()) {
                mf.world.body_mut(c).set_awake(true);
            }
        }
    }

    /// Al destruir una instancia: las demás quitan sus articulaciones hacia ella (el callback de
    /// destrucción) y después la propia instancia destruye las suyas y su cuerpo (onDestroy).
    pub(crate) fn al_destruir_fisica(&mut self, i: usize, pendientes: &[usize]) {
        let uid = self.uid(i);
        for &j in pendientes {
            if j == i {
                continue;
            }
            let Some(f) = fisica_mut(self, j) else {
                continue;
            };
            let mut quedan = Vec::new();
            let mut juntas = Vec::new();
            let mut a_destruir = Vec::new();
            for (k, r) in f.registros.iter().enumerate() {
                let objetivo = match r {
                    RegistroArticulacion::Revolucion { objetivo, .. }
                    | RegistroArticulacion::RevolucionLimitada { objetivo, .. } => *objetivo,
                };
                if objetivo == uid {
                    if let Some(&junta) = f.articulaciones.get(k) {
                        a_destruir.push(junta);
                    }
                } else {
                    quedan.push(*r);
                    if let Some(&junta) = f.articulaciones.get(k) {
                        juntas.push(junta);
                    }
                }
            }
            f.registros = quedan;
            f.articulaciones = juntas;
            f.unidas.retain(|&u| u != uid);
            if let Some(mf) = self.fisica.as_mut() {
                for junta in a_destruir {
                    mf.world.destroy_joint(junta);
                }
            }
        }
        if fisica(self, i).is_some() {
            self.destruir_cuerpo(i);
            if let Some(f) = fisica_mut(self, i) {
                f.registros.clear();
                f.unidas.clear();
            }
        }
    }

    /// Physics.tick: avanza el mundo (la primera instancia del tick) y sincroniza la instancia
    /// con su cuerpo.
    pub(crate) fn tick_fisica(&mut self, i: usize) {
        let Some(mut f) = fisica(self, i).cloned() else {
            return;
        };
        if !f.habilitada {
            return;
        }
        let escala_tiempo = self.escala_tiempo;
        let ticks = self.ticks as i64;
        let dt_real = self.dt;
        let mf = self.mundo_fisico();
        let dt = if mf.paso_fijo {
            escala_tiempo / 60.0
        } else {
            dt_real.min(1.0 / 30.0)
        };
        if ticks > mf.ultimo_tick && escala_tiempo > 0.0 {
            let (vi, pi) = (mf.iteraciones_velocidad, mf.iteraciones_posicion);
            mf.world.step(dt as f32, vi, pi);
            mf.world.clear_forces();
            mf.ultimo_tick = ticks;
            let contactos = std::mem::take(&mut mf.world.begin_contacts);
            for (a, b) in contactos {
                self.registrar_colision_uid(a as u32, b as u32);
            }
        }
        // El tamaño o el cuadro cambiaron: se rehace el cuerpo.
        let (ancho, alto, anim, cuadro) = {
            let s = self.sprite(i);
            (s.ancho, s.alto, s.anim_indice(), s.cuadro as i64)
        };
        if ancho != f.ultimo_ancho
            || alto != f.ultimo_alto
            || Some(anim) != f.ultima_anim
            || cuadro != f.ultimo_cuadro
        {
            *fisica_mut(self, i).expect("física") = f;
            self.crear_cuerpo(i);
            f = fisica(self, i).cloned().expect("física");
            f.ultima_anim = Some(anim);
            f.ultimo_cuadro = cuadro;
        }
        if let Some(cuerpo) = f.cuerpo {
            self.sincronizar(i, cuerpo, &mut f);
        }
        *fisica_mut(self, i).expect("física") = f;
    }

    fn sincronizar(&mut self, i: usize, cuerpo: box2d::BodyId, f: &mut Fisica) {
        let (sx, sy, sa, cx, cy) = {
            let s = self.sprite(i);
            let (cx, cy) = s.centro(&self.proyecto.tipos[s.tipo]);
            (s.x, s.y, s.angulo, cx, cy)
        };
        let movida = sx != f.ultima_x || sy != f.ultima_y;
        let girada = sa != f.ultimo_angulo;
        let world = &mut self.fisica.as_mut().expect("mundo").world;
        if movida {
            let angulo = if girada {
                sa as f32
            } else {
                world.body(cuerpo).get_angle()
            };
            world.set_transform(
                cuerpo,
                b2((ESCALA * cx) as f32, (ESCALA * cy) as f32),
                angulo,
            );
            // La velocidad que deja es el desplazamiento en píxeles (sin pasar a metros).
            world
                .body_mut(cuerpo)
                .set_linear_velocity(b2((cx - f.ultima_x) as f32, (cy - f.ultima_y) as f32));
            f.movida = true;
            world.body_mut(cuerpo).set_awake(true);
        } else if f.movida {
            f.movida = false;
            world.body_mut(cuerpo).set_linear_velocity(b2(0.0, 0.0));
            let angulo = world.body(cuerpo).get_angle();
            world.set_transform(
                cuerpo,
                b2((ESCALA * cx) as f32, (ESCALA * cy) as f32),
                angulo,
            );
        }
        if !movida && girada {
            let p = world.body(cuerpo).get_position();
            world.set_transform(cuerpo, p, sa as f32);
            world.body_mut(cuerpo).set_awake(true);
        }
        let p = world.body(cuerpo).get_position();
        let (nx, ny) = (p.x as f64 / ESCALA, p.y as f64 / ESCALA);
        let na = world.body(cuerpo).get_angle() as f64;
        if nx != sx || ny != sy || na != sa {
            let tipo = self.sprite(i).tipo;
            {
                let s = self.sprite_mut(i);
                s.x = nx;
                s.y = ny;
                s.angulo = na;
            }
            let (cx, cy) = {
                let s = self.sprite(i);
                s.centro(&self.proyecto.tipos[tipo])
            };
            let s = self.sprite_mut(i);
            let (dx, dy) = (cx - s.x, cy - s.y);
            if dx != 0.0 || dy != 0.0 {
                s.x -= dx;
                s.y -= dy;
            }
        }
        let s = self.sprite(i);
        f.ultima_x = s.x;
        f.ultima_y = s.y;
        f.ultimo_angulo = s.angulo;
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn separa_un_poligono_concavo() {
        // Una "L" (en coordenadas de pantalla, el orden del runtime).
        let l = vec![
            dvec2(0.0, 0.0),
            dvec2(10.0, 0.0),
            dvec2(10.0, 4.0),
            dvec2(4.0, 4.0),
            dvec2(4.0, 10.0),
            dvec2(0.0, 10.0),
        ];
        let partes = descomponer(l.clone(), 100.0);
        assert!(partes.len() >= 2, "{partes:?}");
        // Un convexo queda igual.
        let caja = vec![
            dvec2(0.0, 0.0),
            dvec2(10.0, 0.0),
            dvec2(10.0, 10.0),
            dvec2(0.0, 10.0),
        ];
        assert_eq!(descomponer(caja.clone(), 100.0), vec![caja]);
        let _ = l;
    }
}
