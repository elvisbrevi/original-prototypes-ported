// b2Math: vectores, matrices, rotaciones, transformaciones y barridos, en f32 como Box2D.

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::settings::{EPSILON, PI};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Vec2 {
    pub const ZERO: Vec2 = vec2(0.0, 0.0);

    pub fn set_zero(&mut self) {
        *self = Vec2::ZERO;
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Normaliza y devuelve la longitud (0 si es casi nula, sin tocar el vector).
    pub fn normalize(&mut self) -> f32 {
        let length = self.length();
        if length < EPSILON {
            return 0.0;
        }
        let inv = 1.0 / length;
        self.x *= inv;
        self.y *= inv;
        length
    }

    pub fn skew(self) -> Vec2 {
        vec2(-self.y, self.x)
    }

    /// operator(): la componente i.
    pub fn get(self, i: usize) -> f32 {
        if i == 0 { self.x } else { self.y }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, b: Vec2) -> Vec2 {
        vec2(self.x + b.x, self.y + b.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, b: Vec2) -> Vec2 {
        vec2(self.x - b.x, self.y - b.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        vec2(-self.x, -self.y)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, a: Vec2) -> Vec2 {
        vec2(self * a.x, self * a.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, v: Vec2) {
        self.x += v.x;
        self.y += v.y;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, v: Vec2) {
        self.x -= v.x;
        self.y -= v.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, a: f32) {
        self.x *= a;
        self.y *= a;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, b: Vec3) -> Vec3 {
        vec3(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, b: Vec3) -> Vec3 {
        vec3(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        vec3(-self.x, -self.y, -self.z)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, a: Vec3) -> Vec3 {
        vec3(self * a.x, self * a.y, self * a.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, v: Vec3) {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
    }
}

pub fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

pub fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// b2Cross(v, s).
pub fn cross_vs(a: Vec2, s: f32) -> Vec2 {
    vec2(s * a.y, -s * a.x)
}

/// b2Cross(s, v).
pub fn cross_sv(s: f32, a: Vec2) -> Vec2 {
    vec2(-s * a.y, s * a.x)
}

pub fn dot3(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn cross3(a: Vec3, b: Vec3) -> Vec3 {
    vec3(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

pub fn distance(a: Vec2, b: Vec2) -> f32 {
    (a - b).length()
}

pub fn distance_squared(a: Vec2, b: Vec2) -> f32 {
    let c = a - b;
    dot(c, c)
}

pub fn min_v(a: Vec2, b: Vec2) -> Vec2 {
    vec2(min(a.x, b.x), min(a.y, b.y))
}

pub fn max_v(a: Vec2, b: Vec2) -> Vec2 {
    vec2(max(a.x, b.x), max(a.y, b.y))
}

pub fn abs_v(a: Vec2) -> Vec2 {
    vec2(a.x.abs(), a.y.abs())
}

/// b2Min: a < b ? a : b.
pub fn min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

/// b2Max: a > b ? a : b.
pub fn max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

/// b2Clamp: b2Max(low, b2Min(a, high)).
pub fn clamp(a: f32, low: f32, high: f32) -> f32 {
    max(low, min(a, high))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mat22 {
    pub ex: Vec2,
    pub ey: Vec2,
}

impl Mat22 {
    pub fn get_inverse(&self) -> Mat22 {
        let (a, b, c, d) = (self.ex.x, self.ey.x, self.ex.y, self.ey.y);
        let mut det = a * d - b * c;
        if det != 0.0 {
            det = 1.0 / det;
        }
        Mat22 {
            ex: vec2(det * d, -det * c),
            ey: vec2(-det * b, det * a),
        }
    }

    pub fn solve(&self, b: Vec2) -> Vec2 {
        let (a11, a12, a21, a22) = (self.ex.x, self.ey.x, self.ex.y, self.ey.y);
        let mut det = a11 * a22 - a12 * a21;
        if det != 0.0 {
            det = 1.0 / det;
        }
        vec2(det * (a22 * b.x - a12 * b.y), det * (a11 * b.y - a21 * b.x))
    }
}

pub fn mul_mv(a: &Mat22, v: Vec2) -> Vec2 {
    vec2(a.ex.x * v.x + a.ey.x * v.y, a.ex.y * v.x + a.ey.y * v.y)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mat33 {
    pub ex: Vec3,
    pub ey: Vec3,
    pub ez: Vec3,
}

impl Mat33 {
    pub fn solve33(&self, b: Vec3) -> Vec3 {
        let mut det = dot3(self.ex, cross3(self.ey, self.ez));
        if det != 0.0 {
            det = 1.0 / det;
        }
        vec3(
            det * dot3(b, cross3(self.ey, self.ez)),
            det * dot3(self.ex, cross3(b, self.ez)),
            det * dot3(self.ex, cross3(self.ey, b)),
        )
    }

    pub fn solve22(&self, b: Vec2) -> Vec2 {
        let (a11, a12, a21, a22) = (self.ex.x, self.ey.x, self.ex.y, self.ey.y);
        let mut det = a11 * a22 - a12 * a21;
        if det != 0.0 {
            det = 1.0 / det;
        }
        vec2(det * (a22 * b.x - a12 * b.y), det * (a11 * b.y - a21 * b.x))
    }
}

/// b2Mul(Mat33, Vec3).
pub fn mul_m33(a: &Mat33, v: Vec3) -> Vec3 {
    v.x * a.ex + v.y * a.ey + v.z * a.ez
}

/// b2Mul22(Mat33, Vec2).
pub fn mul22(a: &Mat33, v: Vec2) -> Vec2 {
    vec2(a.ex.x * v.x + a.ey.x * v.y, a.ex.y * v.x + a.ey.y * v.y)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rot {
    pub s: f32,
    pub c: f32,
}

impl Default for Rot {
    fn default() -> Self {
        Rot::IDENTITY
    }
}

impl Rot {
    pub const IDENTITY: Rot = Rot { s: 0.0, c: 1.0 };

    pub fn new(angle: f32) -> Rot {
        Rot {
            s: angle.sin(),
            c: angle.cos(),
        }
    }

    pub fn get_angle(self) -> f32 {
        self.s.atan2(self.c)
    }
}

pub fn mul_rr(q: Rot, r: Rot) -> Rot {
    Rot {
        s: q.s * r.c + q.c * r.s,
        c: q.c * r.c - q.s * r.s,
    }
}

pub fn mul_t_rr(q: Rot, r: Rot) -> Rot {
    Rot {
        s: q.c * r.s - q.s * r.c,
        c: q.c * r.c + q.s * r.s,
    }
}

pub fn mul_rv(q: Rot, v: Vec2) -> Vec2 {
    vec2(q.c * v.x - q.s * v.y, q.s * v.x + q.c * v.y)
}

pub fn mul_t_rv(q: Rot, v: Vec2) -> Vec2 {
    vec2(q.c * v.x + q.s * v.y, -q.s * v.x + q.c * v.y)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    pub p: Vec2,
    pub q: Rot,
}

impl Transform {
    pub fn new(p: Vec2, angle: f32) -> Transform {
        Transform {
            p,
            q: Rot::new(angle),
        }
    }
}

pub fn mul_xv(t: &Transform, v: Vec2) -> Vec2 {
    let x = (t.q.c * v.x - t.q.s * v.y) + t.p.x;
    let y = (t.q.s * v.x + t.q.c * v.y) + t.p.y;
    vec2(x, y)
}

pub fn mul_t_xv(t: &Transform, v: Vec2) -> Vec2 {
    let px = v.x - t.p.x;
    let py = v.y - t.p.y;
    vec2(t.q.c * px + t.q.s * py, -t.q.s * px + t.q.c * py)
}

pub fn mul_xx(a: &Transform, b: &Transform) -> Transform {
    Transform {
        q: mul_rr(a.q, b.q),
        p: mul_rv(a.q, b.p) + a.p,
    }
}

pub fn mul_t_xx(a: &Transform, b: &Transform) -> Transform {
    Transform {
        q: mul_t_rr(a.q, b.q),
        p: mul_t_rv(a.q, b.p - a.p),
    }
}

/// b2Sweep: el movimiento de un cuerpo durante el paso, para el impacto continuo (TOI).
#[derive(Clone, Copy, Debug, Default)]
pub struct Sweep {
    pub local_center: Vec2,
    pub c0: Vec2,
    pub c: Vec2,
    pub a0: f32,
    pub a: f32,
    pub alpha0: f32,
}

impl Sweep {
    pub fn get_transform(&self, beta: f32) -> Transform {
        let mut xf = Transform {
            p: (1.0 - beta) * self.c0 + beta * self.c,
            q: Rot::new((1.0 - beta) * self.a0 + beta * self.a),
        };
        xf.p -= mul_rv(xf.q, self.local_center);
        xf
    }

    pub fn advance(&mut self, alpha: f32) {
        let beta = (alpha - self.alpha0) / (1.0 - self.alpha0);
        self.c0 += beta * (self.c - self.c0);
        self.a0 += beta * (self.a - self.a0);
        self.alpha0 = alpha;
    }

    pub fn normalize(&mut self) {
        let two_pi = 2.0 * PI;
        let d = two_pi * (self.a0 / two_pi).floor();
        self.a0 -= d;
        self.a -= d;
    }
}
