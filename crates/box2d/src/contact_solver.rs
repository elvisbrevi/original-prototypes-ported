// b2ContactSolver: impulsos secuenciales con arranque en caliente, el solver de bloque para dos
// puntos y la corrección de posición (también la del sub-paso de impacto continuo).

use crate::collision::{Manifold, ManifoldType, WorldManifold};
use crate::math::{
    Mat22, Rot, Transform, Vec2, clamp, cross, cross_sv, cross_vs, dot, max, min, mul_mv, mul_rv,
    mul_xv, vec2,
};
use crate::settings::{
    BAUMGARTE, LINEAR_SLOP, MAX_LINEAR_CORRECTION, MAX_MANIFOLD_POINTS, TOI_BAUGARTE,
    VELOCITY_THRESHOLD,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct TimeStep {
    pub dt: f32,
    pub inv_dt: f32,
    pub dt_ratio: f32,
    pub velocity_iterations: i32,
    pub position_iterations: i32,
    pub warm_starting: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub c: Vec2,
    pub a: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Velocity {
    pub v: Vec2,
    pub w: f32,
}

/// Lo que el solver necesita de cada contacto de la isla.
pub struct ContactInput {
    pub manifold: Manifold,
    pub friction: f32,
    pub restitution: f32,
    pub tangent_speed: f32,
    pub radius_a: f32,
    pub radius_b: f32,
    pub index_a: usize,
    pub index_b: usize,
    pub inv_mass_a: f32,
    pub inv_mass_b: f32,
    pub inv_i_a: f32,
    pub inv_i_b: f32,
    pub local_center_a: Vec2,
    pub local_center_b: Vec2,
}

#[derive(Clone, Copy, Debug, Default)]
struct VelocityConstraintPoint {
    r_a: Vec2,
    r_b: Vec2,
    normal_impulse: f32,
    tangent_impulse: f32,
    normal_mass: f32,
    tangent_mass: f32,
    velocity_bias: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ContactVelocityConstraint {
    points: [VelocityConstraintPoint; MAX_MANIFOLD_POINTS],
    normal: Vec2,
    normal_mass: Mat22,
    k: Mat22,
    index_a: usize,
    index_b: usize,
    inv_mass_a: f32,
    inv_mass_b: f32,
    inv_i_a: f32,
    inv_i_b: f32,
    friction: f32,
    restitution: f32,
    tangent_speed: f32,
    pub point_count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct ContactPositionConstraint {
    local_points: [Vec2; MAX_MANIFOLD_POINTS],
    local_normal: Vec2,
    local_point: Vec2,
    index_a: usize,
    index_b: usize,
    inv_mass_a: f32,
    inv_mass_b: f32,
    local_center_a: Vec2,
    local_center_b: Vec2,
    inv_i_a: f32,
    inv_i_b: f32,
    kind: ManifoldType,
    radius_a: f32,
    radius_b: f32,
    point_count: usize,
}

pub struct ContactSolver {
    manifolds: Vec<Manifold>,
    velocity_constraints: Vec<ContactVelocityConstraint>,
    position_constraints: Vec<ContactPositionConstraint>,
}

impl ContactSolver {
    pub fn new(step: TimeStep, contacts: &[ContactInput]) -> Self {
        let mut s = Self {
            manifolds: Vec::with_capacity(contacts.len()),
            velocity_constraints: Vec::with_capacity(contacts.len()),
            position_constraints: Vec::with_capacity(contacts.len()),
        };
        for c in contacts {
            let manifold = &c.manifold;
            let point_count = manifold.point_count;
            let mut vc = ContactVelocityConstraint {
                friction: c.friction,
                restitution: c.restitution,
                tangent_speed: c.tangent_speed,
                index_a: c.index_a,
                index_b: c.index_b,
                inv_mass_a: c.inv_mass_a,
                inv_mass_b: c.inv_mass_b,
                inv_i_a: c.inv_i_a,
                inv_i_b: c.inv_i_b,
                point_count,
                ..Default::default()
            };
            let mut pc = ContactPositionConstraint {
                index_a: c.index_a,
                index_b: c.index_b,
                inv_mass_a: c.inv_mass_a,
                inv_mass_b: c.inv_mass_b,
                local_center_a: c.local_center_a,
                local_center_b: c.local_center_b,
                inv_i_a: c.inv_i_a,
                inv_i_b: c.inv_i_b,
                local_normal: manifold.local_normal,
                local_point: manifold.local_point,
                point_count,
                radius_a: c.radius_a,
                radius_b: c.radius_b,
                kind: manifold.kind,
                ..Default::default()
            };
            for j in 0..point_count {
                let cp = &manifold.points[j];
                let vcp = &mut vc.points[j];
                if step.warm_starting {
                    vcp.normal_impulse = step.dt_ratio * cp.normal_impulse;
                    vcp.tangent_impulse = step.dt_ratio * cp.tangent_impulse;
                }
                pc.local_points[j] = cp.local_point;
            }
            s.manifolds.push(*manifold);
            s.velocity_constraints.push(vc);
            s.position_constraints.push(pc);
        }
        s
    }

    pub fn initialize_velocity_constraints(
        &mut self,
        positions: &[Position],
        velocities: &[Velocity],
    ) {
        for (i, vc) in self.velocity_constraints.iter_mut().enumerate() {
            let pc = &self.position_constraints[i];
            let manifold = &self.manifolds[i];
            let (m_a, m_b, i_a, i_b) = (vc.inv_mass_a, vc.inv_mass_b, vc.inv_i_a, vc.inv_i_b);
            let c_a = positions[vc.index_a].c;
            let a_a = positions[vc.index_a].a;
            let v_a = velocities[vc.index_a].v;
            let w_a = velocities[vc.index_a].w;
            let c_b = positions[vc.index_b].c;
            let a_b = positions[vc.index_b].a;
            let v_b = velocities[vc.index_b].v;
            let w_b = velocities[vc.index_b].w;
            let q_a = Rot::new(a_a);
            let q_b = Rot::new(a_b);
            let xf_a = Transform {
                q: q_a,
                p: c_a - mul_rv(q_a, pc.local_center_a),
            };
            let xf_b = Transform {
                q: q_b,
                p: c_b - mul_rv(q_b, pc.local_center_b),
            };
            let mut world_manifold = WorldManifold::default();
            world_manifold.initialize(manifold, &xf_a, pc.radius_a, &xf_b, pc.radius_b);
            vc.normal = world_manifold.normal;
            for j in 0..vc.point_count {
                let vcp = &mut vc.points[j];
                vcp.r_a = world_manifold.points[j] - c_a;
                vcp.r_b = world_manifold.points[j] - c_b;
                let rn_a = cross(vcp.r_a, vc.normal);
                let rn_b = cross(vcp.r_b, vc.normal);
                let k_normal = m_a + m_b + i_a * rn_a * rn_a + i_b * rn_b * rn_b;
                vcp.normal_mass = if k_normal > 0.0 { 1.0 / k_normal } else { 0.0 };
                let tangent = cross_vs(vc.normal, 1.0);
                let rt_a = cross(vcp.r_a, tangent);
                let rt_b = cross(vcp.r_b, tangent);
                let k_tangent = m_a + m_b + i_a * rt_a * rt_a + i_b * rt_b * rt_b;
                vcp.tangent_mass = if k_tangent > 0.0 {
                    1.0 / k_tangent
                } else {
                    0.0
                };
                vcp.velocity_bias = 0.0;
                let v_rel = dot(
                    vc.normal,
                    v_b + cross_sv(w_b, vcp.r_b) - v_a - cross_sv(w_a, vcp.r_a),
                );
                if v_rel < -VELOCITY_THRESHOLD {
                    vcp.velocity_bias = -vc.restitution * v_rel;
                }
            }
            if vc.point_count == 2 {
                let vcp1 = vc.points[0];
                let vcp2 = vc.points[1];
                let rn1_a = cross(vcp1.r_a, vc.normal);
                let rn1_b = cross(vcp1.r_b, vc.normal);
                let rn2_a = cross(vcp2.r_a, vc.normal);
                let rn2_b = cross(vcp2.r_b, vc.normal);
                let k11 = m_a + m_b + i_a * rn1_a * rn1_a + i_b * rn1_b * rn1_b;
                let k22 = m_a + m_b + i_a * rn2_a * rn2_a + i_b * rn2_b * rn2_b;
                let k12 = m_a + m_b + i_a * rn1_a * rn2_a + i_b * rn1_b * rn2_b;
                let k_max_condition_number = 1000.0;
                if k11 * k11 < k_max_condition_number * (k11 * k22 - k12 * k12) {
                    vc.k.ex = vec2(k11, k12);
                    vc.k.ey = vec2(k12, k22);
                    vc.normal_mass = vc.k.get_inverse();
                } else {
                    vc.point_count = 1;
                }
            }
        }
    }

    pub fn warm_start(&self, velocities: &mut [Velocity]) {
        for vc in &self.velocity_constraints {
            let (m_a, i_a, m_b, i_b) = (vc.inv_mass_a, vc.inv_i_a, vc.inv_mass_b, vc.inv_i_b);
            let mut v_a = velocities[vc.index_a].v;
            let mut w_a = velocities[vc.index_a].w;
            let mut v_b = velocities[vc.index_b].v;
            let mut w_b = velocities[vc.index_b].w;
            let normal = vc.normal;
            let tangent = cross_vs(normal, 1.0);
            for vcp in vc.points.iter().take(vc.point_count) {
                let p = vcp.normal_impulse * normal + vcp.tangent_impulse * tangent;
                w_a -= i_a * cross(vcp.r_a, p);
                v_a -= m_a * p;
                w_b += i_b * cross(vcp.r_b, p);
                v_b += m_b * p;
            }
            velocities[vc.index_a].v = v_a;
            velocities[vc.index_a].w = w_a;
            velocities[vc.index_b].v = v_b;
            velocities[vc.index_b].w = w_b;
        }
    }

    pub fn solve_velocity_constraints(&mut self, velocities: &mut [Velocity]) {
        for vc in &mut self.velocity_constraints {
            let (m_a, i_a, m_b, i_b) = (vc.inv_mass_a, vc.inv_i_a, vc.inv_mass_b, vc.inv_i_b);
            let point_count = vc.point_count;
            let mut v_a = velocities[vc.index_a].v;
            let mut w_a = velocities[vc.index_a].w;
            let mut v_b = velocities[vc.index_b].v;
            let mut w_b = velocities[vc.index_b].w;
            let normal = vc.normal;
            let tangent = cross_vs(normal, 1.0);
            let friction = vc.friction;
            // Rozamiento primero: es menos importante que la no penetración.
            for j in 0..point_count {
                let vcp = &mut vc.points[j];
                let dv = v_b + cross_sv(w_b, vcp.r_b) - v_a - cross_sv(w_a, vcp.r_a);
                let vt = dot(dv, tangent) - vc.tangent_speed;
                let mut lambda = vcp.tangent_mass * (-vt);
                let max_friction = friction * vcp.normal_impulse;
                let new_impulse = clamp(vcp.tangent_impulse + lambda, -max_friction, max_friction);
                lambda = new_impulse - vcp.tangent_impulse;
                vcp.tangent_impulse = new_impulse;
                let p = lambda * tangent;
                v_a -= m_a * p;
                w_a -= i_a * cross(vcp.r_a, p);
                v_b += m_b * p;
                w_b += i_b * cross(vcp.r_b, p);
            }
            if point_count == 1 {
                let vcp = &mut vc.points[0];
                let dv = v_b + cross_sv(w_b, vcp.r_b) - v_a - cross_sv(w_a, vcp.r_a);
                let vn = dot(dv, normal);
                let mut lambda = -vcp.normal_mass * (vn - vcp.velocity_bias);
                let new_impulse = max(vcp.normal_impulse + lambda, 0.0);
                lambda = new_impulse - vcp.normal_impulse;
                vcp.normal_impulse = new_impulse;
                let p = lambda * normal;
                v_a -= m_a * p;
                w_a -= i_a * cross(vcp.r_a, p);
                v_b += m_b * p;
                w_b += i_b * cross(vcp.r_b, p);
            } else {
                // Solver de bloque (problema de complementariedad lineal de dos puntos).
                let cp1 = vc.points[0];
                let cp2 = vc.points[1];
                let a = vec2(cp1.normal_impulse, cp2.normal_impulse);
                let dv1 = v_b + cross_sv(w_b, cp1.r_b) - v_a - cross_sv(w_a, cp1.r_a);
                let dv2 = v_b + cross_sv(w_b, cp2.r_b) - v_a - cross_sv(w_a, cp2.r_a);
                let vn1 = dot(dv1, normal);
                let vn2 = dot(dv2, normal);
                let mut b = vec2(vn1 - cp1.velocity_bias, vn2 - cp2.velocity_bias);
                b -= mul_mv(&vc.k, a);
                let aplicar =
                    |x: Vec2, v_a: &mut Vec2, w_a: &mut f32, v_b: &mut Vec2, w_b: &mut f32| {
                        let d = x - a;
                        let p1 = d.x * normal;
                        let p2 = d.y * normal;
                        *v_a -= m_a * (p1 + p2);
                        *w_a -= i_a * (cross(cp1.r_a, p1) + cross(cp2.r_a, p2));
                        *v_b += m_b * (p1 + p2);
                        *w_b += i_b * (cross(cp1.r_b, p1) + cross(cp2.r_b, p2));
                        x
                    };
                let resultado = 'solve: {
                    // Caso 1: los dos puntos activos.
                    let x = -mul_mv(&vc.normal_mass, b);
                    if x.x >= 0.0 && x.y >= 0.0 {
                        break 'solve Some(aplicar(x, &mut v_a, &mut w_a, &mut v_b, &mut w_b));
                    }
                    // Caso 2: solo el primero.
                    let x = vec2(-cp1.normal_mass * b.x, 0.0);
                    let vn2 = vc.k.ex.y * x.x + b.y;
                    if x.x >= 0.0 && vn2 >= 0.0 {
                        break 'solve Some(aplicar(x, &mut v_a, &mut w_a, &mut v_b, &mut w_b));
                    }
                    // Caso 3: solo el segundo.
                    let x = vec2(0.0, -cp2.normal_mass * b.y);
                    let vn1 = vc.k.ey.x * x.y + b.x;
                    if x.y >= 0.0 && vn1 >= 0.0 {
                        break 'solve Some(aplicar(x, &mut v_a, &mut w_a, &mut v_b, &mut w_b));
                    }
                    // Caso 4: ninguno.
                    let x = Vec2::ZERO;
                    if b.x >= 0.0 && b.y >= 0.0 {
                        break 'solve Some(aplicar(x, &mut v_a, &mut w_a, &mut v_b, &mut w_b));
                    }
                    None
                };
                if let Some(x) = resultado {
                    vc.points[0].normal_impulse = x.x;
                    vc.points[1].normal_impulse = x.y;
                }
            }
            velocities[vc.index_a].v = v_a;
            velocities[vc.index_a].w = w_a;
            velocities[vc.index_b].v = v_b;
            velocities[vc.index_b].w = w_b;
        }
    }

    /// StoreImpulses: los impulsos de cada contacto (para el arranque en caliente del paso
    /// siguiente).
    pub fn impulses(&self) -> Vec<[(f32, f32); MAX_MANIFOLD_POINTS]> {
        self.velocity_constraints
            .iter()
            .map(|vc| {
                let mut r = [(0.0, 0.0); MAX_MANIFOLD_POINTS];
                for (j, item) in r.iter_mut().enumerate().take(vc.point_count) {
                    *item = (vc.points[j].normal_impulse, vc.points[j].tangent_impulse);
                }
                r
            })
            .collect()
    }

    /// Cuántos puntos quedaron en cada restricción de velocidad.
    pub fn point_counts(&self) -> Vec<usize> {
        self.velocity_constraints
            .iter()
            .map(|vc| vc.point_count)
            .collect()
    }

    pub fn solve_position_constraints(&self, positions: &mut [Position]) -> bool {
        self.solve_positions(positions, None, BAUMGARTE) >= -3.0 * LINEAR_SLOP
    }

    pub fn solve_toi_position_constraints(
        &self,
        positions: &mut [Position],
        toi_index_a: usize,
        toi_index_b: usize,
    ) -> bool {
        self.solve_positions(positions, Some((toi_index_a, toi_index_b)), TOI_BAUGARTE)
            >= -1.5 * LINEAR_SLOP
    }

    fn solve_positions(
        &self,
        positions: &mut [Position],
        toi: Option<(usize, usize)>,
        baumgarte: f32,
    ) -> f32 {
        let mut min_separation: f32 = 0.0;
        for pc in &self.position_constraints {
            let (index_a, index_b) = (pc.index_a, pc.index_b);
            let (m_a, i_a, m_b, i_b) = match toi {
                None => (pc.inv_mass_a, pc.inv_i_a, pc.inv_mass_b, pc.inv_i_b),
                Some((ta, tb)) => {
                    let (m_a, i_a) = if index_a == ta || index_a == tb {
                        (pc.inv_mass_a, pc.inv_i_a)
                    } else {
                        (0.0, 0.0)
                    };
                    let (m_b, i_b) = if index_b == ta || index_b == tb {
                        (pc.inv_mass_b, pc.inv_i_b)
                    } else {
                        (0.0, 0.0)
                    };
                    (m_a, i_a, m_b, i_b)
                }
            };
            let mut c_a = positions[index_a].c;
            let mut a_a = positions[index_a].a;
            let mut c_b = positions[index_b].c;
            let mut a_b = positions[index_b].a;
            for j in 0..pc.point_count {
                let q_a = Rot::new(a_a);
                let q_b = Rot::new(a_b);
                let xf_a = Transform {
                    q: q_a,
                    p: c_a - mul_rv(q_a, pc.local_center_a),
                };
                let xf_b = Transform {
                    q: q_b,
                    p: c_b - mul_rv(q_b, pc.local_center_b),
                };
                let (normal, point, separation) = position_manifold(pc, &xf_a, &xf_b, j);
                let r_a = point - c_a;
                let r_b = point - c_b;
                min_separation = min(min_separation, separation);
                let c = clamp(
                    baumgarte * (separation + LINEAR_SLOP),
                    -MAX_LINEAR_CORRECTION,
                    0.0,
                );
                let rn_a = cross(r_a, normal);
                let rn_b = cross(r_b, normal);
                let k = m_a + m_b + i_a * rn_a * rn_a + i_b * rn_b * rn_b;
                let impulse = if k > 0.0 { -c / k } else { 0.0 };
                let p = impulse * normal;
                c_a -= m_a * p;
                a_a -= i_a * cross(r_a, p);
                c_b += m_b * p;
                a_b += i_b * cross(r_b, p);
            }
            positions[index_a].c = c_a;
            positions[index_a].a = a_a;
            positions[index_b].c = c_b;
            positions[index_b].a = a_b;
        }
        min_separation
    }
}

/// b2PositionSolverManifold::Initialize.
fn position_manifold(
    pc: &ContactPositionConstraint,
    xf_a: &Transform,
    xf_b: &Transform,
    index: usize,
) -> (Vec2, Vec2, f32) {
    match pc.kind {
        ManifoldType::Circles => {
            let point_a = mul_xv(xf_a, pc.local_point);
            let point_b = mul_xv(xf_b, pc.local_points[0]);
            let mut normal = point_b - point_a;
            normal.normalize();
            let point = 0.5 * (point_a + point_b);
            let separation = dot(point_b - point_a, normal) - pc.radius_a - pc.radius_b;
            (normal, point, separation)
        }
        ManifoldType::FaceA => {
            let normal = mul_rv(xf_a.q, pc.local_normal);
            let plane_point = mul_xv(xf_a, pc.local_point);
            let clip_point = mul_xv(xf_b, pc.local_points[index]);
            let separation = dot(clip_point - plane_point, normal) - pc.radius_a - pc.radius_b;
            (normal, clip_point, separation)
        }
        ManifoldType::FaceB => {
            let normal = mul_rv(xf_b.q, pc.local_normal);
            let plane_point = mul_xv(xf_b, pc.local_point);
            let clip_point = mul_xv(xf_a, pc.local_points[index]);
            let separation = dot(clip_point - plane_point, normal) - pc.radius_a - pc.radius_b;
            (-normal, clip_point, separation)
        }
    }
}
