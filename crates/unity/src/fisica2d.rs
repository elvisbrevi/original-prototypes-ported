// Física 2D de Unity 4.x y 5.x: Box2D (crates/box2d) con un cuerpo por Rigidbody2D y los
// Collider2D como fixtures del Rigidbody2D más cercano hacia arriba en la jerarquía. Los
// colisionadores sin Rigidbody2D son estáticos (aquí, un cuerpo estático por objeto). Antes de
// cada paso se llevan a Box2D los Transform que movió el código (como un SetTransform que además
// despierta al cuerpo) y después del paso los cuerpos escriben su posición en su Transform. Un
// PolygonCollider2D son varias fixtures, una por pieza convexa (ver poligonos.rs). Los
// BeginContact y EndContact de Box2D, agrupados por par de colisionadores (Unity avisa cuando
// empieza a tocarse la primera fixture y cuando deja de tocarse la última), son los
// OnTriggerEnter2D/Exit2D y OnCollisionEnter2D/Exit2D. Con Interpolate, el Transform del cuerpo
// se dibuja entre la pose de antes y la de después del último paso. Como en Unity 5.1 (observado
// en Neuro Battle), dos cuerpos kinematic se tocan si uno de los colisionadores es un trigger. La
// matriz de colisiones por capas es el filtro de Box2D: la capa del objeto como categoría y su
// fila de la matriz como máscara.

use std::collections::{HashMap, HashSet};

use box2d::collision::Aabb;
use box2d::math::Transform as B2Transform;
use box2d::shapes::{CircleShape, PolygonShape};
use box2d::world::{BodyDef, BodyId, BodyType, Filter, FixtureDef, FixtureId, World};
use box2d::{Shape, Vec2 as B2Vec2, vec2 as b2vec2};
use macroquad::prelude::{Mat4, Quat, Vec2, Vec3, vec2, vec3};

use crate::datos::{AjustesFisica2d, MaterialFisico};
use crate::escena::{Escena, Id, Objeto};

/// El PhysicsMaterial2D por defecto de Unity 4.
const MATERIAL_POR_DEFECTO: MaterialFisico = MaterialFisico {
    friccion: 0.4,
    rebote: 0.0,
};

#[derive(Clone, PartialEq, Debug)]
pub enum Forma2D {
    /// BoxCollider2D: el tamaño.
    Caja(Vec2),
    /// CircleCollider2D: el radio.
    Circulo(f32),
    /// PolygonCollider2D: los caminos cerrados.
    Poligono(Vec<Vec<Vec2>>),
}

#[derive(Clone)]
pub struct Colisionador2D {
    pub activo: bool,
    pub trigger: bool,
    pub centro: Vec2,
    pub forma: Forma2D,
    pub material: MaterialFisico,
}

impl Colisionador2D {
    pub fn nuevo(
        activo: bool,
        trigger: bool,
        centro: Vec2,
        forma: Forma2D,
        material: Option<MaterialFisico>,
    ) -> Self {
        Self {
            activo,
            trigger,
            centro,
            forma,
            material: material.unwrap_or(MATERIAL_POR_DEFECTO),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Rigidbody2D {
    pub masa: f32,
    pub arrastre: f32,
    pub arrastre_angular: f32,
    pub escala_gravedad: f32,
    pub angulo_fijo: bool,
    pub cinematico: bool,
    /// RigidbodySleepMode2D: 0 NeverSleep, 1 StartAwake, 2 StartAsleep.
    pub sueno: u32,
    /// RigidbodyInterpolation2D: 0 None, 1 Interpolate (2 Extrapolate no se traduce).
    pub interpolacion: u32,
}

/// OnTriggerEnter2D/Exit2D (si alguno de los dos es trigger) u OnCollisionEnter2D/Exit2D.
#[derive(Clone, Copy, Debug)]
pub struct Contacto2D {
    pub entra: bool,
    pub trigger: bool,
    /// Los objetos de los dos colisionadores.
    pub a: Id,
    pub b: Id,
    /// Los índices de los dos colisionadores en sus objetos (Collision2D.collider).
    pub ca: usize,
    pub cb: usize,
}

/// Un colisionador: el objeto y su índice entre los colisionadores del objeto.
type Clave = (Id, usize);

/// Una pose 2D: posición y ángulo.
type Pose = (Vec2, f32);

/// Lo que define las fixtures de un colisionador: si cambia, se vuelven a crear.
#[derive(Clone, PartialEq, Debug)]
struct Firma {
    cuerpo: BodyId,
    centro: Vec2,
    angulo: f32,
    forma: Forma2D,
    trigger: bool,
    friccion: f32,
    rebote: f32,
    capa: u32,
}

struct FixtureActual {
    fixtures: Vec<FixtureId>,
    firma: Firma,
}

struct CuerpoActual {
    cuerpo: BodyId,
    /// La pose del Transform la última vez que se sincronizó.
    pose: Pose,
    /// La pose del cuerpo antes del último paso (para interpolar).
    anterior: Pose,
}

pub struct Fisica2d {
    pub world: World,
    iteraciones_velocidad: i32,
    iteraciones_posicion: i32,
    rayos_tocan_triggers: bool,
    /// m_LayerCollisionMatrix: con qué capas choca cada capa.
    matriz: Vec<u32>,
    cuerpos: HashMap<Id, CuerpoActual>,
    estaticos: HashMap<Id, CuerpoActual>,
    fixtures: HashMap<Clave, FixtureActual>,
    /// Dato de usuario de cada fixture -> colisionador. Nunca se reutiliza.
    datos: HashMap<usize, Clave>,
    siguiente_dato: usize,
    /// Cuántos pares de fixtures se tocan en cada par de colisionadores.
    toques: HashMap<(Clave, Clave), u32>,
}

impl Fisica2d {
    pub fn nueva(ajustes: &AjustesFisica2d) -> Self {
        let mut world = World::new(b2vec2(ajustes.gravedad[0], ajustes.gravedad[1]));
        world.fixture_events = Some(Vec::new());
        world.sensores_cinematicos = true;
        Self {
            world,
            iteraciones_velocidad: ajustes.iteraciones_velocidad,
            iteraciones_posicion: ajustes.iteraciones_posicion,
            rayos_tocan_triggers: ajustes.rayos_tocan_triggers,
            matriz: ajustes.matriz.clone(),
            cuerpos: HashMap::new(),
            estaticos: HashMap::new(),
            fixtures: HashMap::new(),
            datos: HashMap::new(),
            siguiente_dato: 1,
            toques: HashMap::new(),
        }
    }

    fn quitar_fixtures_de_cuerpo(&mut self, cuerpo: BodyId) {
        self.fixtures.retain(|_, f| f.firma.cuerpo != cuerpo);
    }

    fn quitar_fixture(&mut self, clave: Clave) {
        if let Some(f) = self.fixtures.remove(&clave) {
            for fixture in f.fixtures {
                self.world.destroy_fixture(fixture);
            }
        }
    }

    fn quitar_cuerpo(&mut self, id: Id) {
        if let Some(c) = self.cuerpos.remove(&id) {
            self.quitar_fixtures_de_cuerpo(c.cuerpo);
            self.world.destroy_body(c.cuerpo);
        }
    }

    fn quitar_estatico(&mut self, id: Id) {
        if let Some(c) = self.estaticos.remove(&id) {
            self.quitar_fixtures_de_cuerpo(c.cuerpo);
            self.world.destroy_body(c.cuerpo);
        }
    }

    /// Antes de borrar un objeto: sus colisionadores, su cuerpo y su cuerpo estático.
    pub fn quitar_objeto(&mut self, id: Id, obj: &mut Objeto) {
        for i in 0..obj.colisionadores.len() {
            self.quitar_fixture((id, i));
        }
        self.quitar_cuerpo(id);
        self.quitar_estatico(id);
        // Unity no avisa del final de los contactos de un objeto destruido.
        self.toques.retain(|(a, b), _| a.0 != id && b.0 != id);
    }
}

fn a_b2(v: Vec2) -> B2Vec2 {
    b2vec2(v.x, v.y)
}

fn de_b2(v: B2Vec2) -> Vec2 {
    vec2(v.x, v.y)
}

impl Escena {
    /// La pose 2D (posición y giro en z) del Transform de un objeto.
    fn pose(&self, id: Id) -> Pose {
        let p = self.posicion(id);
        (vec2(p.x, p.y), Escena::angulo_z(self.rotacion(id)))
    }

    /// El Rigidbody2D al que se pega un colisionador: el del objeto o el del padre más cercano.
    fn dueno(&self, id: Id) -> Option<Id> {
        let mut actual = Some(id);
        while let Some(i) = actual {
            if self.fisica.cuerpos.contains_key(&i) {
                return Some(i);
            }
            actual = self.objeto(i).padre;
        }
        None
    }

    /// Crea, mueve o borra los cuerpos y las fixtures según la escena.
    pub fn sincronizar_fisica(&mut self) {
        let ids = self.ids();
        // Rigidbody2D.
        for &id in &ids {
            let obj = self.objeto(id);
            let Some(rb) = obj.cuerpo else { continue };
            if obj.destruido || !self.activo_en_jerarquia(id) {
                self.fisica.quitar_cuerpo(id);
                continue;
            }
            let pose = self.pose(id);
            match self.fisica.cuerpos.get_mut(&id) {
                None => {
                    let def = BodyDef {
                        body_type: if rb.cinematico {
                            BodyType::Kinematic
                        } else {
                            BodyType::Dynamic
                        },
                        position: a_b2(pose.0),
                        angle: pose.1,
                        linear_damping: rb.arrastre,
                        angular_damping: rb.arrastre_angular,
                        gravity_scale: rb.escala_gravedad,
                        fixed_rotation: rb.angulo_fijo,
                        allow_sleep: rb.sueno != 0,
                        awake: rb.sueno != 2,
                        user_data: id,
                        ..Default::default()
                    };
                    let cuerpo = self.fisica.world.create_body(&def);
                    self.fisica.cuerpos.insert(
                        id,
                        CuerpoActual {
                            cuerpo,
                            pose,
                            anterior: pose,
                        },
                    );
                }
                Some(c) if c.pose != pose => {
                    // El código movió el Transform: se teletransporta el cuerpo.
                    let cuerpo = c.cuerpo;
                    c.pose = pose;
                    c.anterior = pose;
                    self.fisica
                        .world
                        .set_transform(cuerpo, a_b2(pose.0), pose.1);
                    self.fisica.world.body_mut(cuerpo).set_awake(true);
                }
                Some(_) => {}
            }
        }
        // Collider2D.
        let mut masas = HashSet::new();
        let mut estaticos_usados = HashSet::new();
        for &id in &ids {
            let n = self.objeto(id).colisionadores.len();
            for i in 0..n {
                let obj = self.objeto(id);
                let col = obj.colisionadores[i].clone();
                if obj.destruido || !col.activo || !self.activo_en_jerarquia(id) {
                    self.fisica.quitar_fixture((id, i));
                    continue;
                }
                let (cuerpo, marco) = match self.dueno(id) {
                    Some(d) => (self.fisica.cuerpos[&d].cuerpo, d),
                    None => {
                        estaticos_usados.insert(id);
                        let pose = self.pose(id);
                        let cuerpo = match self.fisica.estaticos.get_mut(&id) {
                            Some(c) => {
                                if c.pose != pose {
                                    c.pose = pose;
                                    let cuerpo = c.cuerpo;
                                    self.fisica
                                        .world
                                        .set_transform(cuerpo, a_b2(pose.0), pose.1);
                                }
                                c.cuerpo
                            }
                            None => {
                                let cuerpo = self.fisica.world.create_body(&BodyDef {
                                    position: a_b2(pose.0),
                                    angle: pose.1,
                                    user_data: id,
                                    ..Default::default()
                                });
                                self.fisica.estaticos.insert(
                                    id,
                                    CuerpoActual {
                                        cuerpo,
                                        pose,
                                        anterior: pose,
                                    },
                                );
                                cuerpo
                            }
                        };
                        (cuerpo, id)
                    }
                };
                // La forma en el marco del cuerpo: sin escala, con la posición y el giro del dueño.
                let (pos_d, ang_d) = self.pose(marco);
                let marco_cuerpo = Mat4::from_rotation_translation(
                    Quat::from_rotation_z(ang_d),
                    vec3(pos_d.x, pos_d.y, 0.0),
                );
                let relativa = marco_cuerpo.inverse() * self.matriz_mundo(id);
                let c = relativa.transform_point3(vec3(col.centro.x, col.centro.y, 0.0));
                let escala = self.escala(id);
                let mut angulo = Escena::angulo_z(self.rotacion(id)) - ang_d;
                let forma = match &col.forma {
                    Forma2D::Caja(t) => {
                        Forma2D::Caja(vec2((t.x * escala.x).abs(), (t.y * escala.y).abs()))
                    }
                    Forma2D::Circulo(r) => Forma2D::Circulo(r * escala.x.abs().max(escala.y.abs())),
                    // Los puntos (con el centro) pasan enteros al marco del cuerpo, con la
                    // escala y el giro.
                    Forma2D::Poligono(caminos) => {
                        angulo = 0.0;
                        Forma2D::Poligono(
                            caminos
                                .iter()
                                .map(|camino| {
                                    camino
                                        .iter()
                                        .map(|&p| {
                                            let q = relativa.transform_point3(vec3(
                                                p.x + col.centro.x,
                                                p.y + col.centro.y,
                                                0.0,
                                            ));
                                            vec2(q.x, q.y)
                                        })
                                        .collect()
                                })
                                .collect(),
                        )
                    }
                };
                let firma = Firma {
                    cuerpo,
                    centro: vec2(c.x, c.y),
                    angulo,
                    forma,
                    trigger: col.trigger,
                    friccion: col.material.friccion,
                    rebote: col.material.rebote,
                    capa: self.objeto(id).capa,
                };
                if self
                    .fisica
                    .fixtures
                    .get(&(id, i))
                    .is_some_and(|f| f.firma == firma)
                {
                    continue;
                }
                if let Some(vieja) = self.fisica.fixtures.get(&(id, i)) {
                    masas.insert(vieja.firma.cuerpo);
                }
                self.fisica.quitar_fixture((id, i));
                let shapes = match &firma.forma {
                    Forma2D::Caja(t) => {
                        let mut p = PolygonShape::default();
                        p.set_as_box_oriented(
                            t.x / 2.0,
                            t.y / 2.0,
                            a_b2(firma.centro),
                            firma.angulo,
                        );
                        vec![Shape::Polygon(p)]
                    }
                    Forma2D::Circulo(r) => {
                        let mut s = CircleShape::new(*r);
                        s.p = a_b2(firma.centro);
                        vec![Shape::Circle(s)]
                    }
                    Forma2D::Poligono(caminos) => piezas_convexas(caminos),
                };
                let mut fixtures = Vec::with_capacity(shapes.len());
                for shape in shapes {
                    let fixture = self.fisica.world.create_fixture(
                        cuerpo,
                        &FixtureDef {
                            shape,
                            friction: firma.friccion,
                            restitution: firma.rebote,
                            density: 1.0,
                            is_sensor: firma.trigger,
                        },
                    );
                    let dato = self.fisica.siguiente_dato;
                    self.fisica.siguiente_dato += 1;
                    self.fisica.world.set_fixture_user_data(fixture, dato);
                    let capa = firma.capa.min(31);
                    let filtro = Filter {
                        category_bits: 1 << capa,
                        mask_bits: self
                            .fisica
                            .matriz
                            .get(capa as usize)
                            .copied()
                            .unwrap_or(u32::MAX),
                        group_index: 0,
                    };
                    self.fisica.world.set_fixture_filter(fixture, filtro);
                    self.fisica.datos.insert(dato, (id, i));
                    fixtures.push(fixture);
                }
                self.fisica
                    .fixtures
                    .insert((id, i), FixtureActual { fixtures, firma });
                masas.insert(cuerpo);
            }
        }
        // Los cuerpos estáticos que se quedaron sin colisionadores.
        let sobran: Vec<Id> = self
            .fisica
            .estaticos
            .keys()
            .filter(|id| !estaticos_usados.contains(id))
            .copied()
            .collect();
        for id in sobran {
            self.fisica.quitar_estatico(id);
        }
        // Rigidbody2D.mass: Box2D reparte la masa por densidad y Unity la fija.
        for (&id, c) in &self.fisica.cuerpos {
            if !masas.contains(&c.cuerpo) {
                continue;
            }
            let masa = self.objeto(id).cuerpo.as_ref().map_or(1.0, |rb| rb.masa);
            let mut md = self.fisica.world.get_mass_data(c.cuerpo);
            if md.mass > 0.0 {
                md.i *= masa / md.mass;
            }
            md.mass = masa;
            self.fisica.world.set_mass_data(c.cuerpo, &md);
        }
    }

    /// Un paso de Physics2D (después de los FixedUpdate): sincroniza, avanza `dt` y escribe las
    /// poses de los cuerpos en sus Transform. Devuelve los contactos que empezaron o terminaron.
    pub fn paso_fisica(&mut self, dt: f32) -> Vec<Contacto2D> {
        self.sincronizar_fisica();
        let (v, p) = (
            self.fisica.iteraciones_velocidad,
            self.fisica.iteraciones_posicion,
        );
        for c in self.fisica.cuerpos.values_mut() {
            let body = self.fisica.world.body(c.cuerpo);
            c.anterior = (de_b2(body.get_position()), body.get_angle());
        }
        self.fisica.world.step(dt, v, p);
        let cuerpos: Vec<(Id, BodyId)> = self
            .fisica
            .cuerpos
            .iter()
            .map(|(&id, c)| (id, c.cuerpo))
            .collect();
        for (id, cuerpo) in cuerpos {
            let body = self.fisica.world.body(cuerpo);
            if body.body_type == BodyType::Static {
                continue;
            }
            let (pos, ang) = (de_b2(body.get_position()), body.get_angle());
            self.escribir_pose(id, pos, ang);
        }
        let eventos = self
            .fisica
            .world
            .fixture_events
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        let mut contactos = Vec::new();
        for e in eventos {
            let (Some(&(a, ia)), Some(&(b, ib))) = (
                self.fisica.datos.get(&e.user_a),
                self.fisica.datos.get(&e.user_b),
            ) else {
                continue;
            };
            if !self.existe(a) || !self.existe(b) {
                continue;
            }
            let clave = if (a, ia) <= (b, ib) {
                ((a, ia), (b, ib))
            } else {
                ((b, ib), (a, ia))
            };
            let n = self.fisica.toques.entry(clave).or_insert(0);
            if e.begin {
                *n += 1;
                if *n > 1 {
                    continue;
                }
            } else {
                if *n == 0 {
                    continue;
                }
                *n -= 1;
                if *n > 0 {
                    continue;
                }
                self.fisica.toques.remove(&clave);
            }
            let trigger = |s: &Self, id: Id, i: usize| {
                s.objeto(id)
                    .colisionadores
                    .get(i)
                    .is_some_and(|c| c.trigger)
            };
            contactos.push(Contacto2D {
                entra: e.begin,
                trigger: trigger(self, a, ia) || trigger(self, b, ib),
                a,
                b,
                ca: ia,
                cb: ib,
            });
        }
        // Los datos de fixtures ya borradas no vuelven a aparecer.
        let vivos: HashSet<usize> = self
            .fisica
            .fixtures
            .values()
            .flat_map(|f| f.fixtures.iter())
            .map(|&f| self.fisica.world.fixture(f).user_data())
            .collect();
        self.fisica.datos.retain(|d, _| vivos.contains(d));
        contactos
    }

    /// La pose del cuerpo de `id` pasada a su Transform (y recordada para no teletransportarlo).
    fn escribir_pose(&mut self, id: Id, pos: Vec2, ang: f32) {
        let z = self.posicion(id).z;
        self.fijar_posicion(id, vec3(pos.x, pos.y, z));
        if !self
            .objeto(id)
            .cuerpo
            .as_ref()
            .is_some_and(|rb| rb.angulo_fijo)
        {
            let resto = self.rotacion(id);
            let actual = Escena::angulo_z(resto);
            self.fijar_rotacion(id, Quat::from_rotation_z(ang - actual) * resto);
        }
        let pose = self.pose(id);
        if let Some(c) = self.fisica.cuerpos.get_mut(&id) {
            c.pose = pose;
        }
    }

    /// Los cuerpos con Interpolate: la pose de antes del último paso y la actual.
    fn interpolados(&self) -> Vec<(Id, Pose, Pose)> {
        let mut v: Vec<_> = self
            .fisica
            .cuerpos
            .iter()
            .filter(|(id, _)| {
                self.objeto(**id)
                    .cuerpo
                    .as_ref()
                    .is_some_and(|rb| rb.interpolacion == 1)
            })
            .map(|(&id, c)| {
                let body = self.fisica.world.body(c.cuerpo);
                (
                    id,
                    c.anterior,
                    (de_b2(body.get_position()), body.get_angle()),
                )
            })
            .collect();
        v.sort_by_key(|x| x.0);
        v
    }

    /// Antes de los FixedUpdate de un frame: los cuerpos con Interpolate vuelven a tener en su
    /// Transform la pose de la física, la que ven los scripts en FixedUpdate.
    pub fn restaurar_interpolados(&mut self) {
        for (id, _, (pos, ang)) in self.interpolados() {
            self.escribir_pose(id, pos, ang);
        }
    }

    /// Después de los FixedUpdate: los cuerpos con Interpolate se dibujan (y los ve Update) a
    /// una fracción `t` entre la pose de antes del último paso y la de después:
    /// t = (Time.time - Time.fixedTime) / Time.fixedDeltaTime.
    pub fn interpolar(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        for (id, (p0, a0), (p1, a1)) in self.interpolados() {
            self.escribir_pose(id, p0.lerp(p1, t), a0 + (a1 - a0) * t);
        }
    }

    /// Las fixtures (con su colisionador) de las capas de `mascara` que tocan un círculo, en el
    /// orden en que las devuelve el árbol de Box2D.
    fn tocan_circulo(&self, centro: Vec2, radio: f32, mascara: u32) -> Vec<Clave> {
        let aabb = Aabb {
            lower_bound: b2vec2(centro.x - radio, centro.y - radio),
            upper_bound: b2vec2(centro.x + radio, centro.y + radio),
        };
        let mut candidatas = Vec::new();
        self.fisica.world.query_aabb(&aabb, |f| {
            candidatas.push(f);
            true
        });
        let circulo = Shape::Circle(CircleShape::new(radio));
        let xf = B2Transform::new(a_b2(centro), 0.0);
        candidatas
            .into_iter()
            .filter_map(|f| {
                let fixture = self.fisica.world.fixture(f);
                let &(id, i) = self.fisica.datos.get(&fixture.user_data())?;
                let obj = self.objeto(id);
                if mascara & (1 << obj.capa) == 0 {
                    return None;
                }
                if obj.colisionadores[i].trigger && !self.fisica.rayos_tocan_triggers {
                    return None;
                }
                self.fisica
                    .world
                    .test_overlap_fixture(f, &circulo, &xf)
                    .then_some((id, i))
            })
            .collect()
    }

    /// Physics2D.OverlapCircle: el primer colisionador de las capas de `mascara` que toca el
    /// círculo (su objeto).
    pub fn superpone_circulo(&self, centro: Vec2, radio: f32, mascara: u32) -> Option<Id> {
        self.tocan_circulo(centro, radio, mascara)
            .first()
            .map(|c| c.0)
    }

    /// Physics2D.OverlapCircleAll: todos los colisionadores que tocan el círculo (objeto e
    /// índice), una vez cada uno.
    pub fn superpone_circulo_todos(&self, centro: Vec2, radio: f32, mascara: u32) -> Vec<Clave> {
        let mut vistos = Vec::new();
        for c in self.tocan_circulo(centro, radio, mascara) {
            if !vistos.contains(&c) {
                vistos.push(c);
            }
        }
        vistos
    }

    /// Collider2D.bounds: la caja del mundo (mínimo y máximo) que envuelve las fixtures del
    /// colisionador, con la pose actual de su cuerpo.
    pub fn limites(&self, id: Id, i: usize) -> Option<(Vec2, Vec2)> {
        let f = self.fisica.fixtures.get(&(id, i))?;
        let mut caja: Option<(Vec2, Vec2)> = None;
        for &fixture in &f.fixtures {
            let fx = self.fisica.world.fixture(fixture);
            let a = fx.shape.compute_aabb(&self.fisica.world.body(fx.body).xf);
            let (lo, hi) = (de_b2(a.lower_bound), de_b2(a.upper_bound));
            caja = Some(match caja {
                None => (lo, hi),
                Some((l, h)) => (l.min(lo), h.max(hi)),
            });
        }
        caja
    }

    fn cuerpo_de(&self, id: Id) -> Option<BodyId> {
        self.fisica.cuerpos.get(&id).map(|c| c.cuerpo)
    }

    /// rigidbody2D.velocity.
    pub fn velocidad(&self, id: Id) -> Vec2 {
        self.cuerpo_de(id).map_or(Vec2::ZERO, |b| {
            de_b2(self.fisica.world.body(b).linear_velocity)
        })
    }

    /// rigidbody2D.velocity = v.
    pub fn fijar_velocidad(&mut self, id: Id, v: Vec2) {
        if let Some(b) = self.cuerpo_de(id) {
            self.fisica.world.body_mut(b).set_linear_velocity(a_b2(v));
        }
    }

    /// rigidbody2D.AddForce(f) (ForceMode2D.Force).
    pub fn agregar_fuerza(&mut self, id: Id, f: Vec2) {
        if let Some(b) = self.cuerpo_de(id) {
            self.fisica.world.apply_force_to_center(b, a_b2(f), true);
        }
    }

    /// El punto `p` del mundo pasado a 2D.
    pub fn plano(p: Vec3) -> Vec2 {
        vec2(p.x, p.y)
    }
}

/// Las piezas convexas de los caminos de un PolygonCollider2D, ya en el marco del cuerpo, como
/// polígonos de Box2D (con el Set que suelda los vértices cercanos). Las piezas sin área, o que
/// tras soldar quedan con menos de tres vértices, se descartan: Box2D las rechazaría con un
/// b2Assert y las cambiaría por una caja de 2x2.
fn piezas_convexas(caminos: &[Vec<Vec2>]) -> Vec<Shape> {
    let mut shapes = Vec::new();
    for camino in caminos {
        for pieza in crate::poligonos::descomponer(camino) {
            if crate::poligonos::area(&pieza) <= f32::EPSILON {
                continue;
            }
            let puntos: Vec<B2Vec2> = pieza.iter().map(|&p| a_b2(p)).collect();
            let mut p = PolygonShape::default();
            if p.set_soldando(&puntos) {
                shapes.push(Shape::Polygon(p));
            }
        }
    }
    shapes
}
