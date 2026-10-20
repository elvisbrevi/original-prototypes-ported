// b2Shape, b2CircleShape y b2PolygonShape (sin las aristas ni las cadenas, que el comportamiento
// Physics de Construct 2 no crea).

use crate::collision::Aabb;
use crate::math::{Transform, Vec2, cross, cross_vs, dot, max_v, min_v, mul_rv, mul_xv, vec2};
use crate::settings::{MAX_POLYGON_VERTICES, PI, POLYGON_RADIUS};

#[derive(Clone, Copy, Debug, Default)]
pub struct MassData {
    pub mass: f32,
    pub center: Vec2,
    pub i: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct CircleShape {
    pub radius: f32,
    pub p: Vec2,
}

impl CircleShape {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            p: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PolygonShape {
    pub radius: f32,
    pub centroid: Vec2,
    pub vertices: [Vec2; MAX_POLYGON_VERTICES],
    pub normals: [Vec2; MAX_POLYGON_VERTICES],
    pub count: usize,
}

impl Default for PolygonShape {
    fn default() -> Self {
        Self {
            radius: POLYGON_RADIUS,
            centroid: Vec2::ZERO,
            vertices: [Vec2::ZERO; MAX_POLYGON_VERTICES],
            normals: [Vec2::ZERO; MAX_POLYGON_VERTICES],
            count: 0,
        }
    }
}

impl PolygonShape {
    pub fn set_as_box(&mut self, hx: f32, hy: f32) {
        self.count = 4;
        self.vertices[0] = vec2(-hx, -hy);
        self.vertices[1] = vec2(hx, -hy);
        self.vertices[2] = vec2(hx, hy);
        self.vertices[3] = vec2(-hx, hy);
        self.normals[0] = vec2(0.0, -1.0);
        self.normals[1] = vec2(1.0, 0.0);
        self.normals[2] = vec2(0.0, 1.0);
        self.normals[3] = vec2(-1.0, 0.0);
        self.centroid = Vec2::ZERO;
    }

    /// SetAsBox con centro y ángulo (el BoxCollider2D de Unity con su desplazamiento).
    pub fn set_as_box_oriented(&mut self, hx: f32, hy: f32, center: Vec2, angle: f32) {
        self.set_as_box(hx, hy);
        self.centroid = center;
        let xf = Transform::new(center, angle);
        for i in 0..self.count {
            self.vertices[i] = mul_xv(&xf, self.vertices[i]);
            self.normals[i] = mul_rv(xf.q, self.normals[i]);
        }
    }

    /// Set: la envolvente convexa (envoltura de regalo) de los puntos. El box2d.js de Construct 2
    /// tiene comentada la soldadura de vértices cercanos.
    pub fn set(&mut self, vertices: &[Vec2]) {
        let count = vertices.len();
        if count < 3 {
            self.set_as_box(1.0, 1.0);
            return;
        }
        let n = count.min(MAX_POLYGON_VERTICES);
        let ps = &vertices[..n];
        let mut i0 = 0;
        let mut x0 = ps[0].x;
        for (i, p) in ps.iter().enumerate().skip(1) {
            let x = p.x;
            if x > x0 || (x == x0 && p.y < ps[i0].y) {
                i0 = i;
                x0 = x;
            }
        }
        let mut hull = [0usize; MAX_POLYGON_VERTICES];
        let mut m = 0;
        let mut ih = i0;
        loop {
            hull[m] = ih;
            let mut ie = 0;
            for j in 1..n {
                if ie == ih {
                    ie = j;
                    continue;
                }
                let r = ps[ie] - ps[hull[m]];
                let v = ps[j] - ps[hull[m]];
                let c = cross(r, v);
                if c < 0.0 {
                    ie = j;
                }
                if c == 0.0 && v.length_squared() > r.length_squared() {
                    ie = j;
                }
            }
            m += 1;
            ih = ie;
            if ie == i0 || m == MAX_POLYGON_VERTICES {
                break;
            }
        }
        if m < 3 {
            self.set_as_box(1.0, 1.0);
            return;
        }
        self.count = m;
        for i in 0..m {
            self.vertices[i] = ps[hull[i]];
        }
        for i in 0..m {
            let i2 = if i + 1 < m { i + 1 } else { 0 };
            let edge = self.vertices[i2] - self.vertices[i];
            self.normals[i] = cross_vs(edge, 1.0);
            self.normals[i].normalize();
        }
        self.centroid = compute_centroid(&self.vertices[..m]);
    }

    pub fn compute_aabb(&self, xf: &Transform) -> Aabb {
        let mut lower = mul_xv(xf, self.vertices[0]);
        let mut upper = lower;
        for i in 1..self.count {
            let v = mul_xv(xf, self.vertices[i]);
            lower = min_v(lower, v);
            upper = max_v(upper, v);
        }
        let r = vec2(self.radius, self.radius);
        Aabb {
            lower_bound: lower - r,
            upper_bound: upper + r,
        }
    }

    pub fn compute_mass(&self, density: f32) -> MassData {
        let mut center = Vec2::ZERO;
        let mut area = 0.0;
        let mut inertia = 0.0;
        let mut s = Vec2::ZERO;
        for i in 0..self.count {
            s += self.vertices[i];
        }
        s *= 1.0 / self.count as f32;
        let k_inv3 = 1.0 / 3.0;
        for i in 0..self.count {
            let e1 = self.vertices[i] - s;
            let e2 = if i + 1 < self.count {
                self.vertices[i + 1] - s
            } else {
                self.vertices[0] - s
            };
            let d = cross(e1, e2);
            let triangle_area = 0.5 * d;
            area += triangle_area;
            center += triangle_area * k_inv3 * (e1 + e2);
            let (ex1, ey1) = (e1.x, e1.y);
            let (ex2, ey2) = (e2.x, e2.y);
            let intx2 = ex1 * ex1 + ex2 * ex1 + ex2 * ex2;
            let inty2 = ey1 * ey1 + ey2 * ey1 + ey2 * ey2;
            inertia += (0.25 * k_inv3 * d) * (intx2 + inty2);
        }
        let mass = density * area;
        center *= 1.0 / area;
        let mc = center + s;
        let mut i = density * inertia;
        i += mass * (dot(mc, mc) - dot(center, center));
        MassData {
            mass,
            center: mc,
            i,
        }
    }
}

fn compute_centroid(vs: &[Vec2]) -> Vec2 {
    let count = vs.len();
    let mut c = Vec2::ZERO;
    let mut area = 0.0;
    let p_ref = Vec2::ZERO;
    let inv3 = 1.0 / 3.0;
    for i in 0..count {
        let p1 = p_ref;
        let p2 = vs[i];
        let p3 = if i + 1 < count { vs[i + 1] } else { vs[0] };
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let d = cross(e1, e2);
        let triangle_area = 0.5 * d;
        area += triangle_area;
        c += triangle_area * inv3 * (p1 + p2 + p3);
    }
    c *= 1.0 / area;
    c
}

// Como en Box2D, las formas se copian por valor (b2Shape::Clone): no se encajonan.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Circle(CircleShape),
    Polygon(PolygonShape),
}

impl Shape {
    pub fn radius(&self) -> f32 {
        match self {
            Shape::Circle(c) => c.radius,
            Shape::Polygon(p) => p.radius,
        }
    }

    pub fn compute_aabb(&self, xf: &Transform) -> Aabb {
        match self {
            Shape::Circle(c) => {
                let p = xf.p + mul_rv(xf.q, c.p);
                Aabb {
                    lower_bound: vec2(p.x - c.radius, p.y - c.radius),
                    upper_bound: vec2(p.x + c.radius, p.y + c.radius),
                }
            }
            Shape::Polygon(p) => p.compute_aabb(xf),
        }
    }

    pub fn compute_mass(&self, density: f32) -> MassData {
        match self {
            Shape::Circle(c) => {
                let mass = density * PI * c.radius * c.radius;
                MassData {
                    mass,
                    center: c.p,
                    i: mass * (0.5 * c.radius * c.radius + dot(c.p, c.p)),
                }
            }
            Shape::Polygon(p) => p.compute_mass(density),
        }
    }
}
