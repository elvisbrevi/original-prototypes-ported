// b2Distance: distancia entre dos formas convexas con GJK y una caché del símplex.

use crate::math::{
    Transform, Vec2, cross, cross_sv, cross_vs, distance as dist, dot, mul_t_rv, mul_xv,
};
use crate::settings::{EPSILON, MAX_FLOAT, MAX_POLYGON_VERTICES};
use crate::shapes::Shape;

/// b2DistanceProxy (con sus vértices copiados).
#[derive(Clone, Copy, Debug)]
pub struct DistanceProxy {
    pub vertices: [Vec2; MAX_POLYGON_VERTICES],
    pub count: usize,
    pub radius: f32,
}

impl DistanceProxy {
    pub fn new(shape: &Shape) -> Self {
        let mut p = Self {
            vertices: [Vec2::ZERO; MAX_POLYGON_VERTICES],
            count: 0,
            radius: 0.0,
        };
        match shape {
            Shape::Circle(c) => {
                p.vertices[0] = c.p;
                p.count = 1;
                p.radius = c.radius;
            }
            Shape::Polygon(poly) => {
                p.vertices = poly.vertices;
                p.count = poly.count;
                p.radius = poly.radius;
            }
        }
        p
    }

    pub fn get_support(&self, d: Vec2) -> usize {
        let mut best_index = 0;
        let mut best_value = dot(self.vertices[0], d);
        for i in 1..self.count {
            let value = dot(self.vertices[i], d);
            if value > best_value {
                best_index = i;
                best_value = value;
            }
        }
        best_index
    }

    pub fn get_vertex(&self, index: usize) -> Vec2 {
        self.vertices[index]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SimplexCache {
    pub metric: f32,
    pub count: usize,
    pub index_a: [usize; 3],
    pub index_b: [usize; 3],
}

pub struct DistanceInput {
    pub proxy_a: DistanceProxy,
    pub proxy_b: DistanceProxy,
    pub transform_a: Transform,
    pub transform_b: Transform,
    pub use_radii: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DistanceOutput {
    pub point_a: Vec2,
    pub point_b: Vec2,
    pub distance: f32,
    pub iterations: i32,
}

#[derive(Clone, Copy, Debug, Default)]
struct SimplexVertex {
    w_a: Vec2,
    w_b: Vec2,
    w: Vec2,
    a: f32,
    index_a: usize,
    index_b: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Simplex {
    v: [SimplexVertex; 3],
    count: usize,
}

impl Simplex {
    fn read_cache(
        &mut self,
        cache: &SimplexCache,
        proxy_a: &DistanceProxy,
        transform_a: &Transform,
        proxy_b: &DistanceProxy,
        transform_b: &Transform,
    ) {
        self.count = cache.count;
        for i in 0..self.count {
            let v = &mut self.v[i];
            v.index_a = cache.index_a[i];
            v.index_b = cache.index_b[i];
            v.w_a = mul_xv(transform_a, proxy_a.get_vertex(v.index_a));
            v.w_b = mul_xv(transform_b, proxy_b.get_vertex(v.index_b));
            v.w = v.w_b - v.w_a;
            v.a = 0.0;
        }
        if self.count > 1 {
            let metric1 = cache.metric;
            let metric2 = self.get_metric();
            if metric2 < 0.5 * metric1 || 2.0 * metric1 < metric2 || metric2 < EPSILON {
                self.count = 0;
            }
        }
        if self.count == 0 {
            let v = &mut self.v[0];
            v.index_a = 0;
            v.index_b = 0;
            v.w_a = mul_xv(transform_a, proxy_a.get_vertex(0));
            v.w_b = mul_xv(transform_b, proxy_b.get_vertex(0));
            v.w = v.w_b - v.w_a;
            v.a = 1.0;
            self.count = 1;
        }
    }

    fn write_cache(&self, cache: &mut SimplexCache) {
        cache.metric = self.get_metric();
        cache.count = self.count;
        for i in 0..self.count {
            cache.index_a[i] = self.v[i].index_a;
            cache.index_b[i] = self.v[i].index_b;
        }
    }

    fn get_search_direction(&self) -> Vec2 {
        match self.count {
            1 => -self.v[0].w,
            2 => {
                let e12 = self.v[1].w - self.v[0].w;
                let sgn = cross(e12, -self.v[0].w);
                if sgn > 0.0 {
                    cross_sv(1.0, e12)
                } else {
                    cross_vs(e12, 1.0)
                }
            }
            _ => Vec2::ZERO,
        }
    }

    fn get_closest_point(&self) -> Vec2 {
        match self.count {
            1 => self.v[0].w,
            2 => self.v[0].a * self.v[0].w + self.v[1].a * self.v[1].w,
            _ => Vec2::ZERO,
        }
    }

    fn get_witness_points(&self) -> (Vec2, Vec2) {
        let v = &self.v;
        match self.count {
            1 => (v[0].w_a, v[0].w_b),
            2 => (
                v[0].a * v[0].w_a + v[1].a * v[1].w_a,
                v[0].a * v[0].w_b + v[1].a * v[1].w_b,
            ),
            3 => {
                let p = v[0].a * v[0].w_a + v[1].a * v[1].w_a + v[2].a * v[2].w_a;
                (p, p)
            }
            _ => (Vec2::ZERO, Vec2::ZERO),
        }
    }

    fn get_metric(&self) -> f32 {
        let v = &self.v;
        match self.count {
            2 => dist(v[0].w, v[1].w),
            3 => cross(v[1].w - v[0].w, v[2].w - v[0].w),
            _ => 0.0,
        }
    }

    fn solve2(&mut self) {
        let w1 = self.v[0].w;
        let w2 = self.v[1].w;
        let e12 = w2 - w1;
        let d12_2 = -dot(w1, e12);
        if d12_2 <= 0.0 {
            self.v[0].a = 1.0;
            self.count = 1;
            return;
        }
        let d12_1 = dot(w2, e12);
        if d12_1 <= 0.0 {
            self.v[1].a = 1.0;
            self.count = 1;
            self.v[0] = self.v[1];
            return;
        }
        let inv_d12 = 1.0 / (d12_1 + d12_2);
        self.v[0].a = d12_1 * inv_d12;
        self.v[1].a = d12_2 * inv_d12;
        self.count = 2;
    }

    fn solve3(&mut self) {
        let w1 = self.v[0].w;
        let w2 = self.v[1].w;
        let w3 = self.v[2].w;
        let e12 = w2 - w1;
        let w1e12 = dot(w1, e12);
        let w2e12 = dot(w2, e12);
        let d12_1 = w2e12;
        let d12_2 = -w1e12;
        let e13 = w3 - w1;
        let w1e13 = dot(w1, e13);
        let w3e13 = dot(w3, e13);
        let d13_1 = w3e13;
        let d13_2 = -w1e13;
        let e23 = w3 - w2;
        let w2e23 = dot(w2, e23);
        let w3e23 = dot(w3, e23);
        let d23_1 = w3e23;
        let d23_2 = -w2e23;
        let n123 = cross(e12, e13);
        let d123_1 = n123 * cross(w2, w3);
        let d123_2 = n123 * cross(w3, w1);
        let d123_3 = n123 * cross(w1, w2);
        if d12_2 <= 0.0 && d13_2 <= 0.0 {
            self.v[0].a = 1.0;
            self.count = 1;
            return;
        }
        if d12_1 > 0.0 && d12_2 > 0.0 && d123_3 <= 0.0 {
            let inv_d12 = 1.0 / (d12_1 + d12_2);
            self.v[0].a = d12_1 * inv_d12;
            self.v[1].a = d12_2 * inv_d12;
            self.count = 2;
            return;
        }
        if d13_1 > 0.0 && d13_2 > 0.0 && d123_2 <= 0.0 {
            let inv_d13 = 1.0 / (d13_1 + d13_2);
            self.v[0].a = d13_1 * inv_d13;
            self.v[2].a = d13_2 * inv_d13;
            self.count = 2;
            self.v[1] = self.v[2];
            return;
        }
        if d12_1 <= 0.0 && d23_2 <= 0.0 {
            self.v[1].a = 1.0;
            self.count = 1;
            self.v[0] = self.v[1];
            return;
        }
        if d13_1 <= 0.0 && d23_1 <= 0.0 {
            self.v[2].a = 1.0;
            self.count = 1;
            self.v[0] = self.v[2];
            return;
        }
        if d23_1 > 0.0 && d23_2 > 0.0 && d123_1 <= 0.0 {
            let inv_d23 = 1.0 / (d23_1 + d23_2);
            self.v[1].a = d23_1 * inv_d23;
            self.v[2].a = d23_2 * inv_d23;
            self.count = 2;
            self.v[0] = self.v[2];
            return;
        }
        let inv_d123 = 1.0 / (d123_1 + d123_2 + d123_3);
        self.v[0].a = d123_1 * inv_d123;
        self.v[1].a = d123_2 * inv_d123;
        self.v[2].a = d123_3 * inv_d123;
        self.count = 3;
    }
}

/// b2Distance.
pub fn distance(cache: &mut SimplexCache, input: &DistanceInput) -> DistanceOutput {
    let proxy_a = &input.proxy_a;
    let proxy_b = &input.proxy_b;
    let transform_a = input.transform_a;
    let transform_b = input.transform_b;
    let mut simplex = Simplex::default();
    simplex.read_cache(cache, proxy_a, &transform_a, proxy_b, &transform_b);
    const K_MAX_ITERS: i32 = 20;
    let mut save_a = [0usize; 3];
    let mut save_b = [0usize; 3];
    let mut distance_sqr1 = MAX_FLOAT;
    let mut iter = 0;
    while iter < K_MAX_ITERS {
        let save_count = simplex.count;
        for i in 0..save_count {
            save_a[i] = simplex.v[i].index_a;
            save_b[i] = simplex.v[i].index_b;
        }
        match simplex.count {
            2 => simplex.solve2(),
            3 => simplex.solve3(),
            _ => {}
        }
        if simplex.count == 3 {
            break;
        }
        let p = simplex.get_closest_point();
        let distance_sqr2 = p.length_squared();
        // (El original deja vacía la comprobación de que la distancia no aumente.)
        let _ = distance_sqr2 >= distance_sqr1;
        distance_sqr1 = distance_sqr2;
        let d = simplex.get_search_direction();
        if d.length_squared() < EPSILON * EPSILON {
            break;
        }
        let n = simplex.count;
        let vertex = &mut simplex.v[n];
        vertex.index_a = proxy_a.get_support(mul_t_rv(transform_a.q, -d));
        vertex.w_a = mul_xv(&transform_a, proxy_a.get_vertex(vertex.index_a));
        vertex.index_b = proxy_b.get_support(mul_t_rv(transform_b.q, d));
        vertex.w_b = mul_xv(&transform_b, proxy_b.get_vertex(vertex.index_b));
        vertex.w = vertex.w_b - vertex.w_a;
        iter += 1;
        let mut duplicate = false;
        for i in 0..save_count {
            if vertex.index_a == save_a[i] && vertex.index_b == save_b[i] {
                duplicate = true;
                break;
            }
        }
        if duplicate {
            break;
        }
        simplex.count += 1;
    }
    let (point_a, point_b) = simplex.get_witness_points();
    let mut output = DistanceOutput {
        point_a,
        point_b,
        distance: dist(point_a, point_b),
        iterations: iter,
    };
    simplex.write_cache(cache);
    if input.use_radii {
        let r_a = proxy_a.radius;
        let r_b = proxy_b.radius;
        if output.distance > r_a + r_b && output.distance > EPSILON {
            output.distance -= r_a + r_b;
            let mut normal = output.point_b - output.point_a;
            normal.normalize();
            output.point_a += r_a * normal;
            output.point_b -= r_b * normal;
        } else {
            let p = 0.5 * (output.point_a + output.point_b);
            output.point_a = p;
            output.point_b = p;
            output.distance = 0.0;
        }
    }
    output
}
