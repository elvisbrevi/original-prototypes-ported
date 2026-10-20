// b2World, b2Body, b2Fixture, b2Contact, b2ContactManager y b2Island. Las listas enlazadas del
// original (cuerpos, contactos, aristas de contacto y de articulación, fixtures) se mantienen
// con índices: el orden en que se recorren decide el orden de las islas y del solver.

use crate::broad_phase::{BroadPhase, NULL_PROXY};
use crate::collision::{
    Aabb, Manifold, collide_circles, collide_polygon_and_circle, collide_polygons, test_overlap,
};
use crate::contact_solver::{ContactInput, ContactSolver, Position, TimeStep, Velocity};
use crate::distance::DistanceProxy;
use crate::joint::{BodySolverData, RevoluteJoint, RevoluteJointDef};
use crate::math::{Rot, Sweep, Transform, Vec2, cross_sv, dot, min, mul_rv, mul_t_xv, mul_xv};
use crate::settings::{
    ANGULAR_SLEEP_TOLERANCE, EPSILON, LINEAR_SLEEP_TOLERANCE, MAX_FLOAT, MAX_ROTATION,
    MAX_ROTATION_SQUARED, MAX_SUB_STEPS, MAX_TOI_CONTACTS, MAX_TRANSLATION,
    MAX_TRANSLATION_SQUARED, TIME_TO_SLEEP,
};
use crate::shapes::{MassData, Shape};
use crate::toi::{ToiInput, ToiState, time_of_impact};

pub type BodyId = usize;
pub type FixtureId = usize;
pub type ContactRef = usize;
pub type JointId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

const ISLAND_FLAG: u16 = 0x0001;
const AWAKE_FLAG: u16 = 0x0002;
const AUTO_SLEEP_FLAG: u16 = 0x0004;
const BULLET_FLAG: u16 = 0x0008;
const FIXED_ROTATION_FLAG: u16 = 0x0010;
const ACTIVE_FLAG: u16 = 0x0020;

const C_ISLAND_FLAG: u32 = 0x0001;
const C_TOUCHING_FLAG: u32 = 0x0002;
const C_ENABLED_FLAG: u32 = 0x0004;
const C_FILTER_FLAG: u32 = 0x0008;
const C_TOI_FLAG: u32 = 0x0020;

#[derive(Clone, Copy, Debug)]
pub struct BodyDef {
    pub body_type: BodyType,
    pub position: Vec2,
    pub angle: f32,
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub allow_sleep: bool,
    pub awake: bool,
    pub fixed_rotation: bool,
    pub bullet: bool,
    pub active: bool,
    pub gravity_scale: f32,
    pub user_data: usize,
}

impl Default for BodyDef {
    fn default() -> Self {
        Self {
            body_type: BodyType::Static,
            position: Vec2::ZERO,
            angle: 0.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            allow_sleep: true,
            awake: true,
            fixed_rotation: false,
            bullet: false,
            active: true,
            gravity_scale: 1.0,
            user_data: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FixtureDef {
    pub shape: Shape,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub is_sensor: bool,
}

/// Referencia a una arista de la lista de un cuerpo: el contacto (o la articulación) y su lado
/// (0: nodeA/edgeA, 1: nodeB/edgeB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeRef {
    pub id: usize,
    pub side: u8,
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    other: BodyId,
    prev: Option<EdgeRef>,
    next: Option<EdgeRef>,
}

impl Edge {
    fn nuevo(other: BodyId) -> Self {
        Self {
            other,
            prev: None,
            next: None,
        }
    }
}

pub struct Body {
    pub body_type: BodyType,
    flags: u16,
    island_index: usize,
    pub xf: Transform,
    pub sweep: Sweep,
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
    force: Vec2,
    torque: f32,
    prev: Option<BodyId>,
    next: Option<BodyId>,
    fixture_list: Option<FixtureId>,
    joint_list: Option<EdgeRef>,
    contact_list: Option<EdgeRef>,
    mass: f32,
    inv_mass: f32,
    i: f32,
    inv_i: f32,
    linear_damping: f32,
    angular_damping: f32,
    gravity_scale: f32,
    sleep_time: f32,
    pub user_data: usize,
}

impl Fixture {
    pub fn is_sensor(&self) -> bool {
        self.is_sensor
    }

    pub fn user_data(&self) -> usize {
        self.user_data
    }
}

/// BeginContact o EndContact de b2ContactListener, con las dos fixtures y sus datos de usuario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactEvent {
    pub begin: bool,
    pub fixture_a: FixtureId,
    pub fixture_b: FixtureId,
    pub user_a: usize,
    pub user_b: usize,
}

impl Body {
    pub fn is_awake(&self) -> bool {
        self.flags & AWAKE_FLAG != 0
    }

    pub fn is_bullet(&self) -> bool {
        self.flags & BULLET_FLAG != 0
    }

    fn is_active(&self) -> bool {
        self.flags & ACTIVE_FLAG != 0
    }

    /// SetAwake.
    pub fn set_awake(&mut self, flag: bool) {
        if flag {
            if self.flags & AWAKE_FLAG == 0 {
                self.flags |= AWAKE_FLAG;
                self.sleep_time = 0.0;
            }
        } else {
            self.flags &= !AWAKE_FLAG;
            self.sleep_time = 0.0;
            self.linear_velocity = Vec2::ZERO;
            self.angular_velocity = 0.0;
            self.force = Vec2::ZERO;
            self.torque = 0.0;
        }
    }

    pub fn get_position(&self) -> Vec2 {
        self.xf.p
    }

    pub fn get_angle(&self) -> f32 {
        self.sweep.a
    }

    pub fn get_local_center(&self) -> Vec2 {
        self.sweep.local_center
    }

    pub fn get_world_center(&self) -> Vec2 {
        self.sweep.c
    }

    pub fn get_local_point(&self, world_point: Vec2) -> Vec2 {
        mul_t_xv(&self.xf, world_point)
    }

    /// SetLinearVelocity.
    pub fn set_linear_velocity(&mut self, v: Vec2) {
        if self.body_type == BodyType::Static {
            return;
        }
        if dot(v, v) > 0.0 {
            self.set_awake(true);
        }
        self.linear_velocity = v;
    }

    /// SetAngularVelocity.
    pub fn set_angular_velocity(&mut self, w: f32) {
        if self.body_type == BodyType::Static {
            return;
        }
        if w * w > 0.0 {
            self.set_awake(true);
        }
        self.angular_velocity = w;
    }

    fn synchronize_transform(&mut self) {
        self.xf.q = Rot::new(self.sweep.a);
        self.xf.p = self.sweep.c - mul_rv(self.xf.q, self.sweep.local_center);
    }

    fn advance(&mut self, alpha: f32) {
        self.sweep.advance(alpha);
        self.sweep.c = self.sweep.c0;
        self.sweep.a = self.sweep.a0;
        self.synchronize_transform();
    }

    fn solver_data(&self) -> BodySolverData {
        BodySolverData {
            island_index: self.island_index,
            local_center: self.sweep.local_center,
            inv_mass: self.inv_mass,
            inv_i: self.inv_i,
        }
    }
}

pub struct Fixture {
    density: f32,
    /// b2Fixture::m_userData (Unity: el colisionador).
    user_data: usize,
    next: Option<FixtureId>,
    pub body: BodyId,
    pub shape: Shape,
    friction: f32,
    restitution: f32,
    proxy_id: i32,
    proxy_aabb: Aabb,
    proxy_count: usize,
    is_sensor: bool,
}

struct Contact {
    flags: u32,
    prev: Option<ContactRef>,
    next: Option<ContactRef>,
    node_a: Edge,
    node_b: Edge,
    fixture_a: FixtureId,
    fixture_b: FixtureId,
    manifold: Manifold,
    toi_count: i32,
    toi: f32,
    friction: f32,
    restitution: f32,
    tangent_speed: f32,
}

pub struct Joint {
    pub body_a: BodyId,
    pub body_b: BodyId,
    collide_connected: bool,
    island_flag: bool,
    prev: Option<JointId>,
    next: Option<JointId>,
    edge_a: Edge,
    edge_b: Edge,
    pub revolute: RevoluteJoint,
}

/// Arena con huecos reutilizables.
struct Arena<T> {
    items: Vec<Option<T>>,
    free: Vec<usize>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<T> Arena<T> {
    fn insert(&mut self, item: T) -> usize {
        if let Some(i) = self.free.pop() {
            self.items[i] = Some(item);
            i
        } else {
            self.items.push(Some(item));
            self.items.len() - 1
        }
    }

    fn remove(&mut self, i: usize) {
        self.items[i] = None;
        self.free.push(i);
    }

    fn get(&self, i: usize) -> &T {
        self.items[i].as_ref().expect("elemento vivo")
    }

    fn get_mut(&mut self, i: usize) -> &mut T {
        self.items[i].as_mut().expect("elemento vivo")
    }
}

pub struct World {
    bodies: Arena<Body>,
    fixtures: Arena<Fixture>,
    contacts: Arena<Contact>,
    joints: Arena<Joint>,
    body_list: Option<BodyId>,
    joint_list: Option<JointId>,
    body_count: usize,
    joint_count: usize,
    broad_phase: BroadPhase,
    contact_list: Option<ContactRef>,
    contact_count: usize,
    gravity: Vec2,
    allow_sleep: bool,
    warm_starting: bool,
    continuous_physics: bool,
    step_complete: bool,
    new_fixture: bool,
    inv_dt0: f32,
    /// BeginContact: los datos de usuario de los dos cuerpos, en orden.
    pub begin_contacts: Vec<(usize, usize)>,
    /// BeginContact y EndContact por fixture, si se piden (`Some`): los triggers de Unity.
    pub fixture_events: Option<Vec<ContactEvent>>,
}

impl World {
    pub fn new(gravity: Vec2) -> Self {
        Self {
            bodies: Arena::default(),
            fixtures: Arena::default(),
            contacts: Arena::default(),
            joints: Arena::default(),
            body_list: None,
            joint_list: None,
            body_count: 0,
            joint_count: 0,
            broad_phase: BroadPhase::default(),
            contact_list: None,
            contact_count: 0,
            gravity,
            allow_sleep: true,
            warm_starting: true,
            continuous_physics: true,
            step_complete: true,
            new_fixture: false,
            inv_dt0: 0.0,
            begin_contacts: Vec::new(),
            fixture_events: None,
        }
    }

    pub fn fixture(&self, f: FixtureId) -> &Fixture {
        self.fixtures.get(f)
    }

    pub fn set_fixture_user_data(&mut self, f: FixtureId, user_data: usize) {
        self.fixtures.get_mut(f).user_data = user_data;
    }

    /// Las fixtures de un cuerpo, en el orden de su lista (la última creada primero).
    pub fn body_fixtures(&self, b: BodyId) -> Vec<FixtureId> {
        let mut lista = Vec::new();
        let mut f = self.bodies.get(b).fixture_list;
        while let Some(fi) = f {
            lista.push(fi);
            f = self.fixtures.get(fi).next;
        }
        lista
    }

    /// b2Body::ApplyForceToCenter.
    pub fn apply_force_to_center(&mut self, b: BodyId, force: Vec2, wake: bool) {
        let body = self.bodies.get_mut(b);
        if body.body_type != BodyType::Dynamic {
            return;
        }
        if wake && !body.is_awake() {
            body.set_awake(true);
        }
        if body.is_awake() {
            body.force += force;
        }
    }

    /// b2Body::GetMassData.
    pub fn get_mass_data(&self, b: BodyId) -> MassData {
        let body = self.bodies.get(b);
        MassData {
            mass: body.mass,
            center: body.sweep.local_center,
            i: body.i + body.mass * dot(body.sweep.local_center, body.sweep.local_center),
        }
    }

    /// b2Body::SetMassData.
    pub fn set_mass_data(&mut self, b: BodyId, mass_data: &MassData) {
        let body = self.bodies.get_mut(b);
        if body.body_type != BodyType::Dynamic {
            return;
        }
        body.inv_mass = 0.0;
        body.i = 0.0;
        body.inv_i = 0.0;
        body.mass = mass_data.mass;
        if body.mass <= 0.0 {
            body.mass = 1.0;
        }
        body.inv_mass = 1.0 / body.mass;
        if mass_data.i > 0.0 && body.flags & FIXED_ROTATION_FLAG == 0 {
            body.i = mass_data.i - body.mass * dot(mass_data.center, mass_data.center);
            body.inv_i = 1.0 / body.i;
        }
        let old_center = body.sweep.c;
        body.sweep.local_center = mass_data.center;
        body.sweep.c = mul_xv(&body.xf, body.sweep.local_center);
        body.sweep.c0 = body.sweep.c;
        body.linear_velocity += cross_sv(body.angular_velocity, body.sweep.c - old_center);
    }

    /// b2World::QueryAABB: las fixtures cuyo AABB ancho toca `aabb`, hasta que `callback`
    /// devuelva false.
    pub fn query_aabb(&self, aabb: &Aabb, mut callback: impl FnMut(FixtureId) -> bool) {
        self.broad_phase.query(aabb, |proxy_id| {
            callback(self.broad_phase.get_user_data(proxy_id))
        });
    }

    /// b2TestOverlap entre la forma de una fixture (en la posición actual de su cuerpo) y otra.
    pub fn test_overlap_fixture(&self, f: FixtureId, shape: &Shape, xf: &Transform) -> bool {
        let fixture = self.fixtures.get(f);
        let xf_f = self.bodies.get(fixture.body).xf;
        test_overlap(&fixture.shape, &xf_f, shape, xf)
    }

    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    pub fn body(&self, id: BodyId) -> &Body {
        self.bodies.get(id)
    }

    pub fn body_mut(&mut self, id: BodyId) -> &mut Body {
        self.bodies.get_mut(id)
    }

    // --- cuerpos ------------------------------------------------------------------------------

    pub fn create_body(&mut self, bd: &BodyDef) -> BodyId {
        let mut flags = 0;
        if bd.bullet {
            flags |= BULLET_FLAG;
        }
        if bd.fixed_rotation {
            flags |= FIXED_ROTATION_FLAG;
        }
        if bd.allow_sleep {
            flags |= AUTO_SLEEP_FLAG;
        }
        if bd.awake {
            flags |= AWAKE_FLAG;
        }
        if bd.active {
            flags |= ACTIVE_FLAG;
        }
        let xf = Transform::new(bd.position, bd.angle);
        let (mass, inv_mass) = if bd.body_type == BodyType::Dynamic {
            (1.0, 1.0)
        } else {
            (0.0, 0.0)
        };
        let body = Body {
            body_type: bd.body_type,
            flags,
            island_index: 0,
            xf,
            sweep: Sweep {
                local_center: Vec2::ZERO,
                c0: xf.p,
                c: xf.p,
                a0: bd.angle,
                a: bd.angle,
                alpha0: 0.0,
            },
            linear_velocity: bd.linear_velocity,
            angular_velocity: bd.angular_velocity,
            force: Vec2::ZERO,
            torque: 0.0,
            prev: None,
            next: self.body_list,
            fixture_list: None,
            joint_list: None,
            contact_list: None,
            mass,
            inv_mass,
            i: 0.0,
            inv_i: 0.0,
            linear_damping: bd.linear_damping,
            angular_damping: bd.angular_damping,
            gravity_scale: bd.gravity_scale,
            sleep_time: 0.0,
            user_data: bd.user_data,
        };
        let id = self.bodies.insert(body);
        if let Some(head) = self.body_list {
            self.bodies.get_mut(head).prev = Some(id);
        }
        self.body_list = Some(id);
        self.body_count += 1;
        id
    }

    pub fn destroy_body(&mut self, b: BodyId) {
        // Articulaciones.
        let mut je = self.bodies.get(b).joint_list;
        while let Some(e) = je {
            je = self.joint_edge(e).next;
            self.destroy_joint(e.id);
            self.bodies.get_mut(b).joint_list = je;
        }
        self.bodies.get_mut(b).joint_list = None;
        // Contactos.
        let mut ce = self.bodies.get(b).contact_list;
        while let Some(e) = ce {
            ce = self.contact_edge(e).next;
            self.destroy_contact(e.id);
        }
        self.bodies.get_mut(b).contact_list = None;
        // Fixtures.
        let mut f = self.bodies.get(b).fixture_list;
        while let Some(f0) = f {
            f = self.fixtures.get(f0).next;
            self.destroy_proxies(f0);
            self.fixtures.remove(f0);
            self.bodies.get_mut(b).fixture_list = f;
        }
        self.bodies.get_mut(b).fixture_list = None;
        let (prev, next) = {
            let body = self.bodies.get(b);
            (body.prev, body.next)
        };
        if let Some(p) = prev {
            self.bodies.get_mut(p).next = next;
        }
        if let Some(n) = next {
            self.bodies.get_mut(n).prev = prev;
        }
        if self.body_list == Some(b) {
            self.body_list = next;
        }
        self.body_count -= 1;
        self.bodies.remove(b);
    }

    pub fn create_fixture(&mut self, b: BodyId, def: &FixtureDef) -> FixtureId {
        let fixture = Fixture {
            density: def.density,
            next: self.bodies.get(b).fixture_list,
            body: b,
            shape: def.shape,
            friction: def.friction,
            restitution: def.restitution,
            proxy_id: NULL_PROXY,
            proxy_aabb: Aabb::default(),
            proxy_count: 0,
            is_sensor: def.is_sensor,
            user_data: 0,
        };
        let f = self.fixtures.insert(fixture);
        if self.bodies.get(b).is_active() {
            let xf = self.bodies.get(b).xf;
            self.create_proxies(f, &xf);
        }
        self.bodies.get_mut(b).fixture_list = Some(f);
        if def.density > 0.0 {
            self.reset_mass_data(b);
        }
        self.new_fixture = true;
        f
    }

    /// b2Body::DestroyFixture: borra sus contactos (con EndContact si se tocaban), su proxy y
    /// recalcula la masa del cuerpo.
    pub fn destroy_fixture(&mut self, f: FixtureId) {
        let b = self.fixtures.get(f).body;
        // Se quita de la lista simple del cuerpo.
        let mut anterior: Option<FixtureId> = None;
        let mut actual = self.bodies.get(b).fixture_list;
        while let Some(fi) = actual {
            let siguiente = self.fixtures.get(fi).next;
            if fi == f {
                match anterior {
                    Some(a) => self.fixtures.get_mut(a).next = siguiente,
                    None => self.bodies.get_mut(b).fixture_list = siguiente,
                }
                break;
            }
            anterior = Some(fi);
            actual = siguiente;
        }
        let mut edge = self.bodies.get(b).contact_list;
        while let Some(e) = edge {
            edge = self.contact_edge(e).next;
            let c = self.contacts.get(e.id);
            if c.fixture_a == f || c.fixture_b == f {
                self.destroy_contact(e.id);
            }
        }
        if self.bodies.get(b).is_active() {
            self.destroy_proxies(f);
        }
        self.fixtures.remove(f);
        self.reset_mass_data(b);
    }

    fn create_proxies(&mut self, f: FixtureId, xf: &Transform) {
        let aabb = self.fixtures.get(f).shape.compute_aabb(xf);
        let proxy_id = self.broad_phase.create_proxy(&aabb, f);
        let fixture = self.fixtures.get_mut(f);
        fixture.proxy_aabb = aabb;
        fixture.proxy_id = proxy_id;
        fixture.proxy_count = 1;
    }

    fn destroy_proxies(&mut self, f: FixtureId) {
        let fixture = self.fixtures.get_mut(f);
        if fixture.proxy_count == 1 {
            let id = fixture.proxy_id;
            fixture.proxy_id = NULL_PROXY;
            fixture.proxy_count = 0;
            self.broad_phase.destroy_proxy(id);
        }
    }

    fn synchronize_fixture(&mut self, f: FixtureId, xf1: &Transform, xf2: &Transform) {
        let fixture = self.fixtures.get(f);
        if fixture.proxy_count == 0 {
            return;
        }
        let aabb1 = fixture.shape.compute_aabb(xf1);
        let aabb2 = fixture.shape.compute_aabb(xf2);
        let aabb = Aabb::combine(&aabb1, &aabb2);
        let proxy_id = fixture.proxy_id;
        self.fixtures.get_mut(f).proxy_aabb = aabb;
        self.broad_phase.move_proxy(proxy_id, &aabb, xf2.p - xf1.p);
    }

    fn reset_mass_data(&mut self, b: BodyId) {
        let body_type = self.bodies.get(b).body_type;
        {
            let body = self.bodies.get_mut(b);
            body.mass = 0.0;
            body.inv_mass = 0.0;
            body.i = 0.0;
            body.inv_i = 0.0;
            body.sweep.local_center = Vec2::ZERO;
            if body_type != BodyType::Dynamic {
                body.sweep.c0 = body.xf.p;
                body.sweep.c = body.xf.p;
                body.sweep.a0 = body.sweep.a;
                return;
            }
        }
        let mut local_center = Vec2::ZERO;
        let (mut mass, mut inertia) = (0.0, 0.0);
        let mut f = self.bodies.get(b).fixture_list;
        while let Some(fi) = f {
            let fixture = self.fixtures.get(fi);
            f = fixture.next;
            if fixture.density == 0.0 {
                continue;
            }
            let md = fixture.shape.compute_mass(fixture.density);
            mass += md.mass;
            local_center += md.mass * md.center;
            inertia += md.i;
        }
        let body = self.bodies.get_mut(b);
        body.mass = mass;
        body.i = inertia;
        if body.mass > 0.0 {
            body.inv_mass = 1.0 / body.mass;
            local_center *= body.inv_mass;
        } else {
            body.mass = 1.0;
            body.inv_mass = 1.0;
        }
        if body.i > 0.0 && body.flags & FIXED_ROTATION_FLAG == 0 {
            body.i -= body.mass * dot(local_center, local_center);
            body.inv_i = 1.0 / body.i;
        } else {
            body.i = 0.0;
            body.inv_i = 0.0;
        }
        let old_center = body.sweep.c;
        body.sweep.local_center = local_center;
        body.sweep.c = mul_xv(&body.xf, local_center);
        body.sweep.c0 = body.sweep.c;
        body.linear_velocity += cross_sv(body.angular_velocity, body.sweep.c - old_center);
    }

    /// SetTransform.
    pub fn set_transform(&mut self, b: BodyId, position: Vec2, angle: f32) {
        let xf = {
            let body = self.bodies.get_mut(b);
            body.xf.q = Rot::new(angle);
            body.xf.p = position;
            body.sweep.c = mul_xv(&body.xf, body.sweep.local_center);
            body.sweep.a = angle;
            body.sweep.c0 = body.sweep.c;
            body.sweep.a0 = angle;
            body.xf
        };
        let mut f = self.bodies.get(b).fixture_list;
        while let Some(fi) = f {
            f = self.fixtures.get(fi).next;
            self.synchronize_fixture(fi, &xf, &xf);
        }
    }

    fn synchronize_fixtures(&mut self, b: BodyId) {
        let (xf1, xf2) = {
            let body = self.bodies.get(b);
            let q = Rot::new(body.sweep.a0);
            let xf1 = Transform {
                q,
                p: body.sweep.c0 - mul_rv(q, body.sweep.local_center),
            };
            (xf1, body.xf)
        };
        let mut f = self.bodies.get(b).fixture_list;
        while let Some(fi) = f {
            f = self.fixtures.get(fi).next;
            self.synchronize_fixture(fi, &xf1, &xf2);
        }
    }

    /// ShouldCollide (entre cuerpos: al menos uno dinámico y sin articulación que lo impida).
    fn should_collide(&self, b: BodyId, other: BodyId) -> bool {
        let body = self.bodies.get(b);
        if body.body_type != BodyType::Dynamic
            && self.bodies.get(other).body_type != BodyType::Dynamic
        {
            return false;
        }
        let mut jn = body.joint_list;
        while let Some(e) = jn {
            let edge = self.joint_edge(e);
            if edge.other == other && !self.joints.get(e.id).collide_connected {
                return false;
            }
            jn = edge.next;
        }
        true
    }

    // --- articulaciones -----------------------------------------------------------------------

    /// b2RevoluteJointDef::Initialize con el ancla en coordenadas del mundo.
    pub fn revolute_joint_def(
        &self,
        body_a: BodyId,
        body_b: BodyId,
        anchor: Vec2,
    ) -> RevoluteJointDef {
        let a = self.bodies.get(body_a);
        let b = self.bodies.get(body_b);
        RevoluteJointDef {
            body_a,
            body_b,
            collide_connected: false,
            local_anchor_a: a.get_local_point(anchor),
            local_anchor_b: b.get_local_point(anchor),
            reference_angle: b.get_angle() - a.get_angle(),
            enable_limit: false,
            lower_angle: 0.0,
            upper_angle: 0.0,
            enable_motor: false,
            motor_speed: 0.0,
            max_motor_torque: 0.0,
        }
    }

    fn joint_edge(&self, e: EdgeRef) -> &Edge {
        let j = self.joints.get(e.id);
        if e.side == 0 { &j.edge_a } else { &j.edge_b }
    }

    fn joint_edge_mut(&mut self, e: EdgeRef) -> &mut Edge {
        let j = self.joints.get_mut(e.id);
        if e.side == 0 {
            &mut j.edge_a
        } else {
            &mut j.edge_b
        }
    }

    pub fn create_joint(&mut self, def: &RevoluteJointDef) -> JointId {
        let joint = Joint {
            body_a: def.body_a,
            body_b: def.body_b,
            collide_connected: def.collide_connected,
            island_flag: false,
            prev: None,
            next: self.joint_list,
            edge_a: Edge::nuevo(def.body_b),
            edge_b: Edge::nuevo(def.body_a),
            revolute: RevoluteJoint::new(def),
        };
        let j = self.joints.insert(joint);
        if let Some(head) = self.joint_list {
            self.joints.get_mut(head).prev = Some(j);
        }
        self.joint_list = Some(j);
        self.joint_count += 1;
        for (side, body) in [(0u8, def.body_a), (1u8, def.body_b)] {
            let e = EdgeRef { id: j, side };
            let head = self.bodies.get(body).joint_list;
            self.joint_edge_mut(e).next = head;
            if let Some(h) = head {
                self.joint_edge_mut(h).prev = Some(e);
            }
            self.bodies.get_mut(body).joint_list = Some(e);
        }
        if !def.collide_connected {
            self.flag_contacts_between(def.body_b, def.body_a);
        }
        j
    }

    fn flag_contacts_between(&mut self, body_b: BodyId, body_a: BodyId) {
        let mut edge = self.bodies.get(body_b).contact_list;
        while let Some(e) = edge {
            let ce = *self.contact_edge(e);
            if ce.other == body_a {
                self.contacts.get_mut(e.id).flags |= C_FILTER_FLAG;
            }
            edge = ce.next;
        }
    }

    pub fn destroy_joint(&mut self, j: JointId) {
        let (collide_connected, prev, next, body_a, body_b) = {
            let joint = self.joints.get(j);
            (
                joint.collide_connected,
                joint.prev,
                joint.next,
                joint.body_a,
                joint.body_b,
            )
        };
        if let Some(p) = prev {
            self.joints.get_mut(p).next = next;
        }
        if let Some(n) = next {
            self.joints.get_mut(n).prev = prev;
        }
        if self.joint_list == Some(j) {
            self.joint_list = next;
        }
        self.bodies.get_mut(body_a).set_awake(true);
        self.bodies.get_mut(body_b).set_awake(true);
        for (side, body) in [(0u8, body_a), (1u8, body_b)] {
            let e = EdgeRef { id: j, side };
            let edge = *self.joint_edge(e);
            if let Some(p) = edge.prev {
                self.joint_edge_mut(p).next = edge.next;
            }
            if let Some(n) = edge.next {
                self.joint_edge_mut(n).prev = edge.prev;
            }
            if self.bodies.get(body).joint_list == Some(e) {
                self.bodies.get_mut(body).joint_list = edge.next;
            }
        }
        self.joints.remove(j);
        self.joint_count -= 1;
        if !collide_connected {
            self.flag_contacts_between(body_b, body_a);
        }
    }

    // --- contactos ----------------------------------------------------------------------------

    fn contact_edge(&self, e: EdgeRef) -> &Edge {
        let c = self.contacts.get(e.id);
        if e.side == 0 { &c.node_a } else { &c.node_b }
    }

    fn contact_edge_mut(&mut self, e: EdgeRef) -> &mut Edge {
        let c = self.contacts.get_mut(e.id);
        if e.side == 0 {
            &mut c.node_a
        } else {
            &mut c.node_b
        }
    }

    /// b2ContactManager::AddPair.
    fn add_pair(&mut self, fixture_a: FixtureId, fixture_b: FixtureId) {
        let body_a = self.fixtures.get(fixture_a).body;
        let body_b = self.fixtures.get(fixture_b).body;
        if body_a == body_b {
            return;
        }
        let mut edge = self.bodies.get(body_b).contact_list;
        while let Some(e) = edge {
            let ce = self.contact_edge(e);
            if ce.other == body_a {
                let c = self.contacts.get(e.id);
                let (f_a, f_b) = (c.fixture_a, c.fixture_b);
                if (f_a == fixture_a && f_b == fixture_b) || (f_a == fixture_b && f_b == fixture_a)
                {
                    return;
                }
            }
            edge = ce.next;
        }
        if !self.should_collide(body_b, body_a) {
            return;
        }
        // El filtro de Construct 2 deja chocar todo (y el de Box2D, con los filtros por
        // defecto, también).
        // b2Contact::Create: el registro pone el polígono primero en los contactos con círculos.
        let es_circulo =
            |w: &Self, f: FixtureId| matches!(w.fixtures.get(f).shape, Shape::Circle(_));
        let (fixture_a, fixture_b) = if es_circulo(self, fixture_a) && !es_circulo(self, fixture_b)
        {
            (fixture_b, fixture_a)
        } else {
            (fixture_a, fixture_b)
        };
        let fa = self.fixtures.get(fixture_a);
        let fb = self.fixtures.get(fixture_b);
        let body_a = fa.body;
        let body_b = fb.body;
        let contact = Contact {
            flags: C_ENABLED_FLAG,
            prev: None,
            next: self.contact_list,
            node_a: Edge::nuevo(body_b),
            node_b: Edge::nuevo(body_a),
            fixture_a,
            fixture_b,
            manifold: Manifold::default(),
            toi_count: 0,
            toi: 0.0,
            friction: (fa.friction * fb.friction).sqrt(),
            restitution: if fa.restitution > fb.restitution {
                fa.restitution
            } else {
                fb.restitution
            },
            tangent_speed: 0.0,
        };
        let sensores = fa.is_sensor || fb.is_sensor;
        let c = self.contacts.insert(contact);
        if let Some(head) = self.contact_list {
            self.contacts.get_mut(head).prev = Some(c);
        }
        self.contact_list = Some(c);
        for (side, body) in [(0u8, body_a), (1u8, body_b)] {
            let e = EdgeRef { id: c, side };
            let head = self.bodies.get(body).contact_list;
            self.contact_edge_mut(e).next = head;
            if let Some(h) = head {
                self.contact_edge_mut(h).prev = Some(e);
            }
            self.bodies.get_mut(body).contact_list = Some(e);
        }
        if !sensores {
            self.bodies.get_mut(body_a).set_awake(true);
            self.bodies.get_mut(body_b).set_awake(true);
        }
        self.contact_count += 1;
    }

    /// b2ContactManager::Destroy.
    fn destroy_contact(&mut self, c: ContactRef) {
        let (fixture_a, fixture_b, prev, next, point_count) = {
            let contact = self.contacts.get(c);
            (
                contact.fixture_a,
                contact.fixture_b,
                contact.prev,
                contact.next,
                contact.manifold.point_count,
            )
        };
        let body_a = self.fixtures.get(fixture_a).body;
        let body_b = self.fixtures.get(fixture_b).body;
        // EndContact: Construct 2 no hace nada; los triggers de Unity sí.
        if self.contacts.get(c).flags & C_TOUCHING_FLAG != 0 {
            self.record_event(false, fixture_a, fixture_b);
        }
        if let Some(p) = prev {
            self.contacts.get_mut(p).next = next;
        }
        if let Some(n) = next {
            self.contacts.get_mut(n).prev = prev;
        }
        if self.contact_list == Some(c) {
            self.contact_list = next;
        }
        for (side, body) in [(0u8, body_a), (1u8, body_b)] {
            let e = EdgeRef { id: c, side };
            let edge = *self.contact_edge(e);
            if let Some(p) = edge.prev {
                self.contact_edge_mut(p).next = edge.next;
            }
            if let Some(n) = edge.next {
                self.contact_edge_mut(n).prev = edge.prev;
            }
            if self.bodies.get(body).contact_list == Some(e) {
                self.bodies.get_mut(body).contact_list = edge.next;
            }
        }
        // b2Contact::Destroy: despierta a los cuerpos si se tocaban.
        let sensores =
            self.fixtures.get(fixture_a).is_sensor || self.fixtures.get(fixture_b).is_sensor;
        if point_count > 0 && !sensores {
            self.bodies.get_mut(body_a).set_awake(true);
            self.bodies.get_mut(body_b).set_awake(true);
        }
        self.contacts.remove(c);
        self.contact_count -= 1;
    }

    /// b2Contact::Update: el manifold nuevo, con los impulsos de los puntos que siguen.
    fn update_contact(&mut self, c: ContactRef) {
        let (fixture_a, fixture_b, old_manifold, was_touching) = {
            let contact = self.contacts.get_mut(c);
            contact.flags |= C_ENABLED_FLAG;
            (
                contact.fixture_a,
                contact.fixture_b,
                contact.manifold,
                contact.flags & C_TOUCHING_FLAG != 0,
            )
        };
        let fa = self.fixtures.get(fixture_a);
        let fb = self.fixtures.get(fixture_b);
        let sensor = fa.is_sensor || fb.is_sensor;
        let body_a = fa.body;
        let body_b = fb.body;
        let xf_a = self.bodies.get(body_a).xf;
        let xf_b = self.bodies.get(body_b).xf;
        let mut manifold = old_manifold;
        let touching;
        if sensor {
            // Los sensores no generan manifolds.
            touching = test_overlap(&fa.shape, &xf_a, &fb.shape, &xf_b);
            manifold.point_count = 0;
        } else {
            match (&fa.shape, &fb.shape) {
                (Shape::Circle(a), Shape::Circle(b)) => {
                    collide_circles(&mut manifold, a, &xf_a, b, &xf_b)
                }
                (Shape::Polygon(a), Shape::Circle(b)) => {
                    collide_polygon_and_circle(&mut manifold, a, &xf_a, b, &xf_b)
                }
                (Shape::Polygon(a), Shape::Polygon(b)) => {
                    collide_polygons(&mut manifold, a, &xf_a, b, &xf_b)
                }
                (Shape::Circle(_), Shape::Polygon(_)) => {
                    unreachable!("el registro ordena los tipos")
                }
            }
            touching = manifold.point_count > 0;
            for i in 0..manifold.point_count {
                let mp2 = &mut manifold.points[i];
                mp2.normal_impulse = 0.0;
                mp2.tangent_impulse = 0.0;
                let id2 = mp2.id;
                for mp1 in old_manifold.points.iter().take(old_manifold.point_count) {
                    if mp1.id.key() == id2.key() {
                        mp2.normal_impulse = mp1.normal_impulse;
                        mp2.tangent_impulse = mp1.tangent_impulse;
                        break;
                    }
                }
            }
            if touching != was_touching {
                self.bodies.get_mut(body_a).set_awake(true);
                self.bodies.get_mut(body_b).set_awake(true);
            }
        }
        let contact = self.contacts.get_mut(c);
        contact.manifold = manifold;
        if touching {
            contact.flags |= C_TOUCHING_FLAG;
        } else {
            contact.flags &= !C_TOUCHING_FLAG;
        }
        if !was_touching && touching {
            let ua = self.bodies.get(body_a).user_data;
            let ub = self.bodies.get(body_b).user_data;
            self.begin_contacts.push((ua, ub));
            self.record_event(true, fixture_a, fixture_b);
        }
        if was_touching && !touching {
            self.record_event(false, fixture_a, fixture_b);
        }
    }

    fn record_event(&mut self, begin: bool, fixture_a: FixtureId, fixture_b: FixtureId) {
        let user_a = self.fixtures.get(fixture_a).user_data;
        let user_b = self.fixtures.get(fixture_b).user_data;
        if let Some(events) = &mut self.fixture_events {
            events.push(ContactEvent {
                begin,
                fixture_a,
                fixture_b,
                user_a,
                user_b,
            });
        }
    }

    /// b2ContactManager::Collide.
    fn collide(&mut self) {
        let mut c = self.contact_list;
        while let Some(ci) = c {
            let contact = self.contacts.get(ci);
            let (fixture_a, fixture_b, next, flags) = (
                contact.fixture_a,
                contact.fixture_b,
                contact.next,
                contact.flags,
            );
            let body_a = self.fixtures.get(fixture_a).body;
            let body_b = self.fixtures.get(fixture_b).body;
            if flags & C_FILTER_FLAG != 0 {
                if !self.should_collide(body_b, body_a) {
                    c = next;
                    self.destroy_contact(ci);
                    continue;
                }
                self.contacts.get_mut(ci).flags &= !C_FILTER_FLAG;
            }
            let ba = self.bodies.get(body_a);
            let bb = self.bodies.get(body_b);
            let active_a = ba.is_awake() && ba.body_type != BodyType::Static;
            let active_b = bb.is_awake() && bb.body_type != BodyType::Static;
            if !active_a && !active_b {
                c = next;
                continue;
            }
            let proxy_a = self.fixtures.get(fixture_a).proxy_id;
            let proxy_b = self.fixtures.get(fixture_b).proxy_id;
            if !self.broad_phase.test_overlap(proxy_a, proxy_b) {
                c = next;
                self.destroy_contact(ci);
                continue;
            }
            self.update_contact(ci);
            c = self.contacts.get(ci).next;
        }
    }

    fn find_new_contacts(&mut self) {
        for (a, b) in self.broad_phase.update_pairs() {
            self.add_pair(a, b);
        }
    }

    // --- paso ---------------------------------------------------------------------------------

    /// b2World::Step.
    pub fn step(&mut self, dt: f32, velocity_iterations: i32, position_iterations: i32) {
        if self.new_fixture {
            self.find_new_contacts();
            self.new_fixture = false;
        }
        let step = TimeStep {
            dt,
            velocity_iterations,
            position_iterations,
            inv_dt: if dt > 0.0 { 1.0 / dt } else { 0.0 },
            dt_ratio: self.inv_dt0 * dt,
            warm_starting: self.warm_starting,
        };
        self.collide();
        if self.step_complete && step.dt > 0.0 {
            self.solve(&step);
        }
        if self.continuous_physics && step.dt > 0.0 {
            self.solve_toi(&step);
        }
        if step.dt > 0.0 {
            self.inv_dt0 = step.inv_dt;
        }
        self.clear_forces();
    }

    pub fn clear_forces(&mut self) {
        let mut b = self.body_list;
        while let Some(bi) = b {
            let body = self.bodies.get_mut(bi);
            body.force = Vec2::ZERO;
            body.torque = 0.0;
            b = body.next;
        }
    }

    fn solve(&mut self, step: &TimeStep) {
        let mut island = Island::new(self.body_count, self.contact_count, self.joint_count);
        let mut b = self.body_list;
        while let Some(bi) = b {
            let body = self.bodies.get_mut(bi);
            body.flags &= !ISLAND_FLAG;
            b = body.next;
        }
        let mut c = self.contact_list;
        while let Some(ci) = c {
            let contact = self.contacts.get_mut(ci);
            contact.flags &= !C_ISLAND_FLAG;
            c = contact.next;
        }
        let mut j = self.joint_list;
        while let Some(ji) = j {
            let joint = self.joints.get_mut(ji);
            joint.island_flag = false;
            j = joint.next;
        }
        let mut stack: Vec<BodyId> = Vec::with_capacity(self.body_count);
        let mut seed = self.body_list;
        while let Some(si) = seed {
            seed = self.bodies.get(si).next;
            let s = self.bodies.get(si);
            if s.flags & ISLAND_FLAG != 0 {
                continue;
            }
            if !s.is_awake() || !s.is_active() {
                continue;
            }
            if s.body_type == BodyType::Static {
                continue;
            }
            island.clear();
            stack.clear();
            stack.push(si);
            self.bodies.get_mut(si).flags |= ISLAND_FLAG;
            while let Some(bi) = stack.pop() {
                island.add_body(&mut self.bodies, bi);
                self.bodies.get_mut(bi).set_awake(true);
                if self.bodies.get(bi).body_type == BodyType::Static {
                    continue;
                }
                let mut ce = self.bodies.get(bi).contact_list;
                while let Some(e) = ce {
                    let edge = *self.contact_edge(e);
                    ce = edge.next;
                    let contact = self.contacts.get(e.id);
                    if contact.flags & C_ISLAND_FLAG != 0 {
                        continue;
                    }
                    if contact.flags & C_ENABLED_FLAG == 0 || contact.flags & C_TOUCHING_FLAG == 0 {
                        continue;
                    }
                    if self.fixtures.get(contact.fixture_a).is_sensor
                        || self.fixtures.get(contact.fixture_b).is_sensor
                    {
                        continue;
                    }
                    island.contacts.push(e.id);
                    self.contacts.get_mut(e.id).flags |= C_ISLAND_FLAG;
                    let other = edge.other;
                    if self.bodies.get(other).flags & ISLAND_FLAG != 0 {
                        continue;
                    }
                    stack.push(other);
                    self.bodies.get_mut(other).flags |= ISLAND_FLAG;
                }
                let mut je = self.bodies.get(bi).joint_list;
                while let Some(e) = je {
                    let edge = *self.joint_edge(e);
                    je = edge.next;
                    if self.joints.get(e.id).island_flag {
                        continue;
                    }
                    let other = edge.other;
                    if !self.bodies.get(other).is_active() {
                        continue;
                    }
                    island.joints.push(e.id);
                    self.joints.get_mut(e.id).island_flag = true;
                    if self.bodies.get(other).flags & ISLAND_FLAG != 0 {
                        continue;
                    }
                    stack.push(other);
                    self.bodies.get_mut(other).flags |= ISLAND_FLAG;
                }
            }
            self.solve_island(&mut island, step);
            for &bi in &island.bodies {
                let body = self.bodies.get_mut(bi);
                if body.body_type == BodyType::Static {
                    body.flags &= !ISLAND_FLAG;
                }
            }
        }
        let mut b = self.body_list;
        while let Some(bi) = b {
            let body = self.bodies.get(bi);
            b = body.next;
            if body.flags & ISLAND_FLAG == 0 || body.body_type == BodyType::Static {
                continue;
            }
            self.synchronize_fixtures(bi);
        }
        self.find_new_contacts();
    }

    fn contact_inputs(&self, contacts: &[ContactRef]) -> Vec<ContactInput> {
        contacts
            .iter()
            .map(|&ci| {
                let c = self.contacts.get(ci);
                let fa = self.fixtures.get(c.fixture_a);
                let fb = self.fixtures.get(c.fixture_b);
                let ba = self.bodies.get(fa.body);
                let bb = self.bodies.get(fb.body);
                ContactInput {
                    manifold: c.manifold,
                    friction: c.friction,
                    restitution: c.restitution,
                    tangent_speed: c.tangent_speed,
                    radius_a: fa.shape.radius(),
                    radius_b: fb.shape.radius(),
                    index_a: ba.island_index,
                    index_b: bb.island_index,
                    inv_mass_a: ba.inv_mass,
                    inv_mass_b: bb.inv_mass,
                    inv_i_a: ba.inv_i,
                    inv_i_b: bb.inv_i,
                    local_center_a: ba.sweep.local_center,
                    local_center_b: bb.sweep.local_center,
                }
            })
            .collect()
    }

    fn store_impulses(&mut self, island: &Island, solver: &ContactSolver) {
        let impulses = solver.impulses();
        let counts = solver.point_counts();
        for (k, &ci) in island.contacts.iter().enumerate() {
            let manifold = &mut self.contacts.get_mut(ci).manifold;
            for (point, &(normal, tangent)) in
                manifold.points.iter_mut().zip(&impulses[k]).take(counts[k])
            {
                point.normal_impulse = normal;
                point.tangent_impulse = tangent;
            }
        }
    }

    /// b2Island::Solve.
    fn solve_island(&mut self, island: &mut Island, step: &TimeStep) {
        let h = step.dt;
        let gravity = self.gravity;
        island.positions.clear();
        island.velocities.clear();
        for &bi in &island.bodies {
            let b = self.bodies.get_mut(bi);
            let c = b.sweep.c;
            let a = b.sweep.a;
            let mut v = b.linear_velocity;
            let mut w = b.angular_velocity;
            b.sweep.c0 = b.sweep.c;
            b.sweep.a0 = b.sweep.a;
            if b.body_type == BodyType::Dynamic {
                v += h * (b.gravity_scale * gravity + b.inv_mass * b.force);
                w += h * b.inv_i * b.torque;
                v *= 1.0 / (1.0 + h * b.linear_damping);
                w *= 1.0 / (1.0 + h * b.angular_damping);
            }
            island.positions.push(Position { c, a });
            island.velocities.push(Velocity { v, w });
        }
        let inputs = self.contact_inputs(&island.contacts);
        let mut solver = ContactSolver::new(*step, &inputs);
        solver.initialize_velocity_constraints(&island.positions, &island.velocities);
        if step.warm_starting {
            solver.warm_start(&mut island.velocities);
        }
        for &ji in &island.joints {
            let (a, b) = {
                let j = self.joints.get(ji);
                (
                    self.bodies.get(j.body_a).solver_data(),
                    self.bodies.get(j.body_b).solver_data(),
                )
            };
            self.joints.get_mut(ji).revolute.init_velocity_constraints(
                &a,
                &b,
                step,
                &island.positions,
                &mut island.velocities,
            );
        }
        for _ in 0..step.velocity_iterations {
            for &ji in &island.joints {
                self.joints
                    .get_mut(ji)
                    .revolute
                    .solve_velocity_constraints(step, &mut island.velocities);
            }
            solver.solve_velocity_constraints(&mut island.velocities);
        }
        self.store_impulses(island, &solver);
        integrate(&mut island.positions, &mut island.velocities, h);
        let mut position_solved = false;
        for _ in 0..step.position_iterations {
            let contacts_okay = solver.solve_position_constraints(&mut island.positions);
            let mut joints_okay = true;
            for &ji in &island.joints {
                let joint_okay = self
                    .joints
                    .get(ji)
                    .revolute
                    .solve_position_constraints(&mut island.positions);
                joints_okay = joints_okay && joint_okay;
            }
            if contacts_okay && joints_okay {
                position_solved = true;
                break;
            }
        }
        for (i, &bi) in island.bodies.iter().enumerate() {
            let body = self.bodies.get_mut(bi);
            body.sweep.c = island.positions[i].c;
            body.sweep.a = island.positions[i].a;
            body.linear_velocity = island.velocities[i].v;
            body.angular_velocity = island.velocities[i].w;
            body.synchronize_transform();
        }
        if self.allow_sleep {
            let mut min_sleep_time = MAX_FLOAT;
            let lin_tol_sqr = LINEAR_SLEEP_TOLERANCE * LINEAR_SLEEP_TOLERANCE;
            let ang_tol_sqr = ANGULAR_SLEEP_TOLERANCE * ANGULAR_SLEEP_TOLERANCE;
            for &bi in &island.bodies {
                let b = self.bodies.get_mut(bi);
                if b.body_type == BodyType::Static {
                    continue;
                }
                if b.flags & AUTO_SLEEP_FLAG == 0
                    || b.angular_velocity * b.angular_velocity > ang_tol_sqr
                    || dot(b.linear_velocity, b.linear_velocity) > lin_tol_sqr
                {
                    b.sleep_time = 0.0;
                    min_sleep_time = 0.0;
                } else {
                    b.sleep_time += h;
                    min_sleep_time = min(min_sleep_time, b.sleep_time);
                }
            }
            if min_sleep_time >= TIME_TO_SLEEP && position_solved {
                for &bi in &island.bodies {
                    self.bodies.get_mut(bi).set_awake(false);
                }
            }
        }
    }

    /// b2World::SolveTOI: sub-pasos hasta el primer impacto de cada contacto continuo.
    fn solve_toi(&mut self, step: &TimeStep) {
        let mut island = Island::new(2 * MAX_TOI_CONTACTS, MAX_TOI_CONTACTS, 0);
        if self.step_complete {
            let mut b = self.body_list;
            while let Some(bi) = b {
                let body = self.bodies.get_mut(bi);
                body.flags &= !ISLAND_FLAG;
                body.sweep.alpha0 = 0.0;
                b = body.next;
            }
            let mut c = self.contact_list;
            while let Some(ci) = c {
                let contact = self.contacts.get_mut(ci);
                contact.flags &= !(C_TOI_FLAG | C_ISLAND_FLAG);
                contact.toi_count = 0;
                contact.toi = 1.0;
                c = contact.next;
            }
        }
        loop {
            let mut min_contact = None;
            let mut min_alpha = 1.0;
            let mut c = self.contact_list;
            while let Some(ci) = c {
                c = self.contacts.get(ci).next;
                let contact = self.contacts.get(ci);
                if contact.flags & C_ENABLED_FLAG == 0 {
                    continue;
                }
                if contact.toi_count > MAX_SUB_STEPS {
                    continue;
                }
                let alpha;
                if contact.flags & C_TOI_FLAG != 0 {
                    alpha = contact.toi;
                } else {
                    let fa = self.fixtures.get(contact.fixture_a);
                    let fb = self.fixtures.get(contact.fixture_b);
                    if fa.is_sensor || fb.is_sensor {
                        continue;
                    }
                    let (ba_id, bb_id) = (fa.body, fb.body);
                    let ba = self.bodies.get(ba_id);
                    let bb = self.bodies.get(bb_id);
                    let active_a = ba.is_awake() && ba.body_type != BodyType::Static;
                    let active_b = bb.is_awake() && bb.body_type != BodyType::Static;
                    if !active_a && !active_b {
                        continue;
                    }
                    let collide_a = ba.is_bullet() || ba.body_type != BodyType::Dynamic;
                    let collide_b = bb.is_bullet() || bb.body_type != BodyType::Dynamic;
                    if !collide_a && !collide_b {
                        continue;
                    }
                    let mut alpha0 = ba.sweep.alpha0;
                    if ba.sweep.alpha0 < bb.sweep.alpha0 {
                        alpha0 = bb.sweep.alpha0;
                        self.bodies.get_mut(ba_id).sweep.advance(alpha0);
                    } else if bb.sweep.alpha0 < ba.sweep.alpha0 {
                        alpha0 = ba.sweep.alpha0;
                        self.bodies.get_mut(bb_id).sweep.advance(alpha0);
                    }
                    let input = ToiInput {
                        proxy_a: DistanceProxy::new(&fa.shape),
                        proxy_b: DistanceProxy::new(&fb.shape),
                        sweep_a: self.bodies.get(ba_id).sweep,
                        sweep_b: self.bodies.get(bb_id).sweep,
                        t_max: 1.0,
                    };
                    let output = time_of_impact(&input);
                    let beta = output.t;
                    alpha = if output.state == ToiState::Touching {
                        min(alpha0 + (1.0 - alpha0) * beta, 1.0)
                    } else {
                        1.0
                    };
                    let contact = self.contacts.get_mut(ci);
                    contact.toi = alpha;
                    contact.flags |= C_TOI_FLAG;
                }
                if alpha < min_alpha {
                    min_contact = Some(ci);
                    min_alpha = alpha;
                }
            }
            let Some(mc) = min_contact.filter(|_| 1.0 - 10.0 * EPSILON >= min_alpha) else {
                self.step_complete = true;
                break;
            };
            let ba_id = self.fixtures.get(self.contacts.get(mc).fixture_a).body;
            let bb_id = self.fixtures.get(self.contacts.get(mc).fixture_b).body;
            let backup1 = self.bodies.get(ba_id).sweep;
            let backup2 = self.bodies.get(bb_id).sweep;
            self.bodies.get_mut(ba_id).advance(min_alpha);
            self.bodies.get_mut(bb_id).advance(min_alpha);
            self.update_contact(mc);
            {
                let contact = self.contacts.get_mut(mc);
                contact.flags &= !C_TOI_FLAG;
                contact.toi_count += 1;
            }
            let flags = self.contacts.get(mc).flags;
            if flags & C_ENABLED_FLAG == 0 || flags & C_TOUCHING_FLAG == 0 {
                self.contacts.get_mut(mc).flags &= !C_ENABLED_FLAG;
                let a = self.bodies.get_mut(ba_id);
                a.sweep = backup1;
                a.synchronize_transform();
                let b = self.bodies.get_mut(bb_id);
                b.sweep = backup2;
                b.synchronize_transform();
                continue;
            }
            self.bodies.get_mut(ba_id).set_awake(true);
            self.bodies.get_mut(bb_id).set_awake(true);
            island.clear();
            island.add_body(&mut self.bodies, ba_id);
            island.add_body(&mut self.bodies, bb_id);
            island.contacts.push(mc);
            self.bodies.get_mut(ba_id).flags |= ISLAND_FLAG;
            self.bodies.get_mut(bb_id).flags |= ISLAND_FLAG;
            self.contacts.get_mut(mc).flags |= C_ISLAND_FLAG;
            for body in [ba_id, bb_id] {
                if self.bodies.get(body).body_type != BodyType::Dynamic {
                    continue;
                }
                let mut ce = self.bodies.get(body).contact_list;
                while let Some(e) = ce {
                    if island.bodies.len() == island.body_capacity {
                        break;
                    }
                    if island.contacts.len() == island.contact_capacity {
                        break;
                    }
                    let edge = *self.contact_edge(e);
                    ce = edge.next;
                    let contact = self.contacts.get(e.id);
                    if contact.flags & C_ISLAND_FLAG != 0 {
                        continue;
                    }
                    let other = edge.other;
                    let other_body = self.bodies.get(other);
                    if other_body.body_type == BodyType::Dynamic
                        && !self.bodies.get(body).is_bullet()
                        && !other_body.is_bullet()
                    {
                        continue;
                    }
                    if self.fixtures.get(contact.fixture_a).is_sensor
                        || self.fixtures.get(contact.fixture_b).is_sensor
                    {
                        continue;
                    }
                    let backup = other_body.sweep;
                    if other_body.flags & ISLAND_FLAG == 0 {
                        self.bodies.get_mut(other).advance(min_alpha);
                    }
                    self.update_contact(e.id);
                    let flags = self.contacts.get(e.id).flags;
                    if flags & C_ENABLED_FLAG == 0 || flags & C_TOUCHING_FLAG == 0 {
                        let o = self.bodies.get_mut(other);
                        o.sweep = backup;
                        o.synchronize_transform();
                        continue;
                    }
                    self.contacts.get_mut(e.id).flags |= C_ISLAND_FLAG;
                    island.contacts.push(e.id);
                    if self.bodies.get(other).flags & ISLAND_FLAG != 0 {
                        continue;
                    }
                    let o = self.bodies.get_mut(other);
                    o.flags |= ISLAND_FLAG;
                    if o.body_type != BodyType::Static {
                        o.set_awake(true);
                    }
                    island.add_body(&mut self.bodies, other);
                }
            }
            let dt = (1.0 - min_alpha) * step.dt;
            let sub_step = TimeStep {
                dt,
                inv_dt: 1.0 / dt,
                dt_ratio: 1.0,
                position_iterations: 20,
                velocity_iterations: step.velocity_iterations,
                warm_starting: false,
            };
            let toi_a = self.bodies.get(ba_id).island_index;
            let toi_b = self.bodies.get(bb_id).island_index;
            self.solve_island_toi(&mut island, &sub_step, toi_a, toi_b);
            for &bi in &island.bodies {
                let body = self.bodies.get_mut(bi);
                body.flags &= !ISLAND_FLAG;
                if body.body_type != BodyType::Dynamic {
                    continue;
                }
                self.synchronize_fixtures(bi);
                let mut ce = self.bodies.get(bi).contact_list;
                while let Some(e) = ce {
                    ce = self.contact_edge(e).next;
                    self.contacts.get_mut(e.id).flags &= !(C_TOI_FLAG | C_ISLAND_FLAG);
                }
            }
            self.find_new_contacts();
            // Sin sub-pasos (m_subStepping es false).
        }
    }

    /// b2Island::SolveTOI.
    fn solve_island_toi(
        &mut self,
        island: &mut Island,
        sub_step: &TimeStep,
        toi_a: usize,
        toi_b: usize,
    ) {
        island.positions.clear();
        island.velocities.clear();
        for &bi in &island.bodies {
            let b = self.bodies.get(bi);
            island.positions.push(Position {
                c: b.sweep.c,
                a: b.sweep.a,
            });
            island.velocities.push(Velocity {
                v: b.linear_velocity,
                w: b.angular_velocity,
            });
        }
        let inputs = self.contact_inputs(&island.contacts);
        let mut solver = ContactSolver::new(*sub_step, &inputs);
        for _ in 0..sub_step.position_iterations {
            if solver.solve_toi_position_constraints(&mut island.positions, toi_a, toi_b) {
                break;
            }
        }
        for idx in [toi_a, toi_b] {
            let bi = island.bodies[idx];
            let b = self.bodies.get_mut(bi);
            b.sweep.c0 = island.positions[idx].c;
            b.sweep.a0 = island.positions[idx].a;
        }
        solver.initialize_velocity_constraints(&island.positions, &island.velocities);
        for _ in 0..sub_step.velocity_iterations {
            solver.solve_velocity_constraints(&mut island.velocities);
        }
        let h = sub_step.dt;
        integrate(&mut island.positions, &mut island.velocities, h);
        for (i, &bi) in island.bodies.iter().enumerate() {
            let body = self.bodies.get_mut(bi);
            body.sweep.c = island.positions[i].c;
            body.sweep.a = island.positions[i].a;
            body.linear_velocity = island.velocities[i].v;
            body.angular_velocity = island.velocities[i].w;
            body.synchronize_transform();
        }
    }
}

/// La integración de posiciones de b2Island, con los límites de traslación y rotación por paso.
fn integrate(positions: &mut [Position], velocities: &mut [Velocity], h: f32) {
    for (p, vel) in positions.iter_mut().zip(velocities.iter_mut()) {
        let mut c = p.c;
        let mut a = p.a;
        let mut v = vel.v;
        let mut w = vel.w;
        let translation = h * v;
        if dot(translation, translation) > MAX_TRANSLATION_SQUARED {
            let ratio = MAX_TRANSLATION / translation.length();
            v *= ratio;
        }
        let rotation = h * w;
        if rotation * rotation > MAX_ROTATION_SQUARED {
            let ratio = MAX_ROTATION / rotation.abs();
            w *= ratio;
        }
        c += h * v;
        a += h * w;
        p.c = c;
        p.a = a;
        vel.v = v;
        vel.w = w;
    }
}

struct Island {
    bodies: Vec<BodyId>,
    contacts: Vec<ContactRef>,
    joints: Vec<JointId>,
    positions: Vec<Position>,
    velocities: Vec<Velocity>,
    body_capacity: usize,
    contact_capacity: usize,
}

impl Island {
    fn new(body_capacity: usize, contact_capacity: usize, joint_capacity: usize) -> Self {
        Self {
            bodies: Vec::with_capacity(body_capacity),
            contacts: Vec::with_capacity(contact_capacity),
            joints: Vec::with_capacity(joint_capacity),
            positions: Vec::with_capacity(body_capacity),
            velocities: Vec::with_capacity(body_capacity),
            body_capacity,
            contact_capacity,
        }
    }

    fn clear(&mut self) {
        self.bodies.clear();
        self.contacts.clear();
        self.joints.clear();
    }

    fn add_body(&mut self, bodies: &mut Arena<Body>, b: BodyId) {
        bodies.get_mut(b).island_index = self.bodies.len();
        self.bodies.push(b);
    }
}
