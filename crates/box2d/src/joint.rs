// b2Joint con b2RevoluteJoint, la única articulación que crea el comportamiento Physics en los
// juegos del museo (Create revolute joint y Create limited revolute joint).

use crate::contact_solver::{Position, TimeStep, Velocity};
use crate::math::{Mat33, Rot, Vec2, Vec3, clamp, cross, cross_sv, mul_rv, vec2, vec3};
use crate::settings::{ANGULAR_SLOP, LINEAR_SLOP, MAX_ANGULAR_CORRECTION};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitState {
    Inactive,
    AtLower,
    AtUpper,
    Equal,
}

/// b2RevoluteJointDef (Initialize lo hace el mundo, que conoce los cuerpos).
#[derive(Clone, Copy, Debug)]
pub struct RevoluteJointDef {
    pub body_a: usize,
    pub body_b: usize,
    pub collide_connected: bool,
    pub local_anchor_a: Vec2,
    pub local_anchor_b: Vec2,
    pub reference_angle: f32,
    pub enable_limit: bool,
    pub lower_angle: f32,
    pub upper_angle: f32,
    pub enable_motor: bool,
    pub motor_speed: f32,
    pub max_motor_torque: f32,
}

/// Lo que la articulación toma de cada cuerpo al iniciar el paso.
#[derive(Clone, Copy, Debug, Default)]
pub struct BodySolverData {
    pub island_index: usize,
    pub local_center: Vec2,
    pub inv_mass: f32,
    pub inv_i: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RevoluteJoint {
    pub local_anchor_a: Vec2,
    pub local_anchor_b: Vec2,
    impulse: Vec3,
    motor_impulse: f32,
    pub enable_motor: bool,
    pub max_motor_torque: f32,
    pub motor_speed: f32,
    pub enable_limit: bool,
    pub reference_angle: f32,
    pub lower_angle: f32,
    pub upper_angle: f32,
    index_a: usize,
    index_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    local_center_a: Vec2,
    local_center_b: Vec2,
    inv_mass_a: f32,
    inv_mass_b: f32,
    inv_i_a: f32,
    inv_i_b: f32,
    mass: Mat33,
    motor_mass: f32,
    limit_state: LimitState,
}

impl RevoluteJoint {
    pub fn new(def: &RevoluteJointDef) -> Self {
        Self {
            local_anchor_a: def.local_anchor_a,
            local_anchor_b: def.local_anchor_b,
            reference_angle: def.reference_angle,
            impulse: Vec3::default(),
            motor_impulse: 0.0,
            lower_angle: def.lower_angle,
            upper_angle: def.upper_angle,
            max_motor_torque: def.max_motor_torque,
            motor_speed: def.motor_speed,
            enable_limit: def.enable_limit,
            enable_motor: def.enable_motor,
            limit_state: LimitState::Inactive,
            index_a: 0,
            index_b: 0,
            r_a: Vec2::ZERO,
            r_b: Vec2::ZERO,
            local_center_a: Vec2::ZERO,
            local_center_b: Vec2::ZERO,
            inv_mass_a: 0.0,
            inv_mass_b: 0.0,
            inv_i_a: 0.0,
            inv_i_b: 0.0,
            mass: Mat33::default(),
            motor_mass: 0.0,
        }
    }

    pub fn init_velocity_constraints(
        &mut self,
        a: &BodySolverData,
        b: &BodySolverData,
        step: &TimeStep,
        positions: &[Position],
        velocities: &mut [Velocity],
    ) {
        self.index_a = a.island_index;
        self.index_b = b.island_index;
        self.local_center_a = a.local_center;
        self.local_center_b = b.local_center;
        self.inv_mass_a = a.inv_mass;
        self.inv_mass_b = b.inv_mass;
        self.inv_i_a = a.inv_i;
        self.inv_i_b = b.inv_i;
        let a_a = positions[self.index_a].a;
        let mut v_a = velocities[self.index_a].v;
        let mut w_a = velocities[self.index_a].w;
        let a_b = positions[self.index_b].a;
        let mut v_b = velocities[self.index_b].v;
        let mut w_b = velocities[self.index_b].w;
        let q_a = Rot::new(a_a);
        let q_b = Rot::new(a_b);
        self.r_a = mul_rv(q_a, self.local_anchor_a - self.local_center_a);
        self.r_b = mul_rv(q_b, self.local_anchor_b - self.local_center_b);
        let (m_a, m_b) = (self.inv_mass_a, self.inv_mass_b);
        let (i_a, i_b) = (self.inv_i_a, self.inv_i_b);
        let fixed_rotation = i_a + i_b == 0.0;
        let (r_a, r_b) = (self.r_a, self.r_b);
        self.mass.ex.x = m_a + m_b + r_a.y * r_a.y * i_a + r_b.y * r_b.y * i_b;
        self.mass.ey.x = -r_a.y * r_a.x * i_a - r_b.y * r_b.x * i_b;
        self.mass.ez.x = -r_a.y * i_a - r_b.y * i_b;
        self.mass.ex.y = self.mass.ey.x;
        self.mass.ey.y = m_a + m_b + r_a.x * r_a.x * i_a + r_b.x * r_b.x * i_b;
        self.mass.ez.y = r_a.x * i_a + r_b.x * i_b;
        self.mass.ex.z = self.mass.ez.x;
        self.mass.ey.z = self.mass.ez.y;
        self.mass.ez.z = i_a + i_b;
        self.motor_mass = i_a + i_b;
        if self.motor_mass > 0.0 {
            self.motor_mass = 1.0 / self.motor_mass;
        }
        if !self.enable_motor || fixed_rotation {
            self.motor_impulse = 0.0;
        }
        if self.enable_limit && !fixed_rotation {
            let joint_angle = a_b - a_a - self.reference_angle;
            if (self.upper_angle - self.lower_angle).abs() < 2.0 * ANGULAR_SLOP {
                self.limit_state = LimitState::Equal;
            } else if joint_angle <= self.lower_angle {
                if self.limit_state != LimitState::AtLower {
                    self.impulse.z = 0.0;
                }
                self.limit_state = LimitState::AtLower;
            } else if joint_angle >= self.upper_angle {
                if self.limit_state != LimitState::AtUpper {
                    self.impulse.z = 0.0;
                }
                self.limit_state = LimitState::AtUpper;
            } else {
                self.limit_state = LimitState::Inactive;
                self.impulse.z = 0.0;
            }
        } else {
            self.limit_state = LimitState::Inactive;
        }
        if step.warm_starting {
            self.impulse = step.dt_ratio * self.impulse;
            self.motor_impulse *= step.dt_ratio;
            let p = vec2(self.impulse.x, self.impulse.y);
            v_a -= m_a * p;
            w_a -= i_a * (cross(self.r_a, p) + self.motor_impulse + self.impulse.z);
            v_b += m_b * p;
            w_b += i_b * (cross(self.r_b, p) + self.motor_impulse + self.impulse.z);
        } else {
            self.impulse = Vec3::default();
            self.motor_impulse = 0.0;
        }
        velocities[self.index_a].v = v_a;
        velocities[self.index_a].w = w_a;
        velocities[self.index_b].v = v_b;
        velocities[self.index_b].w = w_b;
    }

    pub fn solve_velocity_constraints(&mut self, step: &TimeStep, velocities: &mut [Velocity]) {
        let mut v_a = velocities[self.index_a].v;
        let mut w_a = velocities[self.index_a].w;
        let mut v_b = velocities[self.index_b].v;
        let mut w_b = velocities[self.index_b].w;
        let (m_a, m_b) = (self.inv_mass_a, self.inv_mass_b);
        let (i_a, i_b) = (self.inv_i_a, self.inv_i_b);
        let fixed_rotation = i_a + i_b == 0.0;
        if self.enable_motor && self.limit_state != LimitState::Equal && !fixed_rotation {
            let c_dot = w_b - w_a - self.motor_speed;
            let mut impulse = -self.motor_mass * c_dot;
            let old_impulse = self.motor_impulse;
            let max_impulse = step.dt * self.max_motor_torque;
            self.motor_impulse = clamp(self.motor_impulse + impulse, -max_impulse, max_impulse);
            impulse = self.motor_impulse - old_impulse;
            w_a -= i_a * impulse;
            w_b += i_b * impulse;
        }
        if self.enable_limit && self.limit_state != LimitState::Inactive && !fixed_rotation {
            let c_dot1 = v_b + cross_sv(w_b, self.r_b) - v_a - cross_sv(w_a, self.r_a);
            let c_dot2 = w_b - w_a;
            let c_dot = vec3(c_dot1.x, c_dot1.y, c_dot2);
            let mut impulse = -self.mass.solve33(c_dot);
            match self.limit_state {
                LimitState::Equal => self.impulse += impulse,
                LimitState::AtLower | LimitState::AtUpper => {
                    let new_impulse = self.impulse.z + impulse.z;
                    let fuera = if self.limit_state == LimitState::AtLower {
                        new_impulse < 0.0
                    } else {
                        new_impulse > 0.0
                    };
                    if fuera {
                        let rhs = -c_dot1 + self.impulse.z * vec2(self.mass.ez.x, self.mass.ez.y);
                        let reduced = self.mass.solve22(rhs);
                        impulse.x = reduced.x;
                        impulse.y = reduced.y;
                        impulse.z = -self.impulse.z;
                        self.impulse.x += reduced.x;
                        self.impulse.y += reduced.y;
                        self.impulse.z = 0.0;
                    } else {
                        self.impulse += impulse;
                    }
                }
                LimitState::Inactive => {}
            }
            let p = vec2(impulse.x, impulse.y);
            v_a -= m_a * p;
            w_a -= i_a * (cross(self.r_a, p) + impulse.z);
            v_b += m_b * p;
            w_b += i_b * (cross(self.r_b, p) + impulse.z);
        } else {
            let c_dot = v_b + cross_sv(w_b, self.r_b) - v_a - cross_sv(w_a, self.r_a);
            let impulse = self.mass.solve22(-c_dot);
            self.impulse.x += impulse.x;
            self.impulse.y += impulse.y;
            v_a -= m_a * impulse;
            w_a -= i_a * cross(self.r_a, impulse);
            v_b += m_b * impulse;
            w_b += i_b * cross(self.r_b, impulse);
        }
        velocities[self.index_a].v = v_a;
        velocities[self.index_a].w = w_a;
        velocities[self.index_b].v = v_b;
        velocities[self.index_b].w = w_b;
    }

    pub fn solve_position_constraints(&self, positions: &mut [Position]) -> bool {
        let mut c_a = positions[self.index_a].c;
        let mut a_a = positions[self.index_a].a;
        let mut c_b = positions[self.index_b].c;
        let mut a_b = positions[self.index_b].a;
        let mut angular_error = 0.0;
        let fixed_rotation = self.inv_i_a + self.inv_i_b == 0.0;
        if self.enable_limit && self.limit_state != LimitState::Inactive && !fixed_rotation {
            let angle = a_b - a_a - self.reference_angle;
            let mut limit_impulse = 0.0;
            match self.limit_state {
                LimitState::Equal => {
                    let c = clamp(
                        angle - self.lower_angle,
                        -MAX_ANGULAR_CORRECTION,
                        MAX_ANGULAR_CORRECTION,
                    );
                    limit_impulse = -self.motor_mass * c;
                    angular_error = c.abs();
                }
                LimitState::AtLower => {
                    let c = angle - self.lower_angle;
                    angular_error = -c;
                    let c = clamp(c + ANGULAR_SLOP, -MAX_ANGULAR_CORRECTION, 0.0);
                    limit_impulse = -self.motor_mass * c;
                }
                LimitState::AtUpper => {
                    let c = angle - self.upper_angle;
                    angular_error = c;
                    let c = clamp(c - ANGULAR_SLOP, 0.0, MAX_ANGULAR_CORRECTION);
                    limit_impulse = -self.motor_mass * c;
                }
                LimitState::Inactive => {}
            }
            a_a -= self.inv_i_a * limit_impulse;
            a_b += self.inv_i_b * limit_impulse;
        }
        let q_a = Rot::new(a_a);
        let q_b = Rot::new(a_b);
        let r_a = mul_rv(q_a, self.local_anchor_a - self.local_center_a);
        let r_b = mul_rv(q_b, self.local_anchor_b - self.local_center_b);
        let c = c_b + r_b - c_a - r_a;
        let position_error = c.length();
        let (m_a, m_b) = (self.inv_mass_a, self.inv_mass_b);
        let (i_a, i_b) = (self.inv_i_a, self.inv_i_b);
        let mut k = crate::math::Mat22::default();
        k.ex.x = m_a + m_b + i_a * r_a.y * r_a.y + i_b * r_b.y * r_b.y;
        k.ex.y = -i_a * r_a.x * r_a.y - i_b * r_b.x * r_b.y;
        k.ey.x = k.ex.y;
        k.ey.y = m_a + m_b + i_a * r_a.x * r_a.x + i_b * r_b.x * r_b.x;
        let impulse = -k.solve(c);
        c_a -= m_a * impulse;
        a_a -= i_a * cross(r_a, impulse);
        c_b += m_b * impulse;
        a_b += i_b * cross(r_b, impulse);
        positions[self.index_a].c = c_a;
        positions[self.index_a].a = a_a;
        positions[self.index_b].c = c_b;
        positions[self.index_b].a = a_b;
        position_error <= LINEAR_SLOP && angular_error <= ANGULAR_SLOP
    }
}
