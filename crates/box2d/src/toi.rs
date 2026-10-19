// b2TimeOfImpact: el instante en que dos formas que se mueven (barridos) se tocan, por avance
// conservador con una función de separación.

use crate::distance::{DistanceInput, DistanceProxy, SimplexCache, distance};
use crate::math::{Sweep, Vec2, cross_vs, dot, max, mul_rv, mul_t_rv, mul_xv};
use crate::settings::{LINEAR_SLOP, MAX_POLYGON_VERTICES};

pub struct ToiInput {
    pub proxy_a: DistanceProxy,
    pub proxy_b: DistanceProxy,
    pub sweep_a: Sweep,
    pub sweep_b: Sweep,
    pub t_max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToiState {
    Unknown,
    Failed,
    Overlapped,
    Touching,
    Separated,
}

#[derive(Clone, Copy, Debug)]
pub struct ToiOutput {
    pub state: ToiState,
    pub t: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SepType {
    Points,
    FaceA,
    FaceB,
}

struct SeparationFunction<'a> {
    proxy_a: &'a DistanceProxy,
    proxy_b: &'a DistanceProxy,
    sweep_a: Sweep,
    sweep_b: Sweep,
    kind: SepType,
    local_point: Vec2,
    axis: Vec2,
}

impl<'a> SeparationFunction<'a> {
    fn initialize(
        cache: &SimplexCache,
        proxy_a: &'a DistanceProxy,
        sweep_a: Sweep,
        proxy_b: &'a DistanceProxy,
        sweep_b: Sweep,
        t1: f32,
    ) -> Self {
        let mut f = Self {
            proxy_a,
            proxy_b,
            sweep_a,
            sweep_b,
            kind: SepType::Points,
            local_point: Vec2::ZERO,
            axis: Vec2::ZERO,
        };
        let xf_a = f.sweep_a.get_transform(t1);
        let xf_b = f.sweep_b.get_transform(t1);
        if cache.count == 1 {
            f.kind = SepType::Points;
            let point_a = mul_xv(&xf_a, proxy_a.get_vertex(cache.index_a[0]));
            let point_b = mul_xv(&xf_b, proxy_b.get_vertex(cache.index_b[0]));
            f.axis = point_b - point_a;
            f.axis.normalize();
        } else if cache.index_a[0] == cache.index_a[1] {
            f.kind = SepType::FaceB;
            let lb1 = proxy_b.get_vertex(cache.index_b[0]);
            let lb2 = proxy_b.get_vertex(cache.index_b[1]);
            f.axis = cross_vs(lb2 - lb1, 1.0);
            f.axis.normalize();
            let normal = mul_rv(xf_b.q, f.axis);
            f.local_point = 0.5 * (lb1 + lb2);
            let point_b = mul_xv(&xf_b, f.local_point);
            let point_a = mul_xv(&xf_a, proxy_a.get_vertex(cache.index_a[0]));
            let s = dot(point_a - point_b, normal);
            if s < 0.0 {
                f.axis = -f.axis;
            }
        } else {
            f.kind = SepType::FaceA;
            let la1 = proxy_a.get_vertex(cache.index_a[0]);
            let la2 = proxy_a.get_vertex(cache.index_a[1]);
            f.axis = cross_vs(la2 - la1, 1.0);
            f.axis.normalize();
            let normal = mul_rv(xf_a.q, f.axis);
            f.local_point = 0.5 * (la1 + la2);
            let point_a = mul_xv(&xf_a, f.local_point);
            let point_b = mul_xv(&xf_b, proxy_b.get_vertex(cache.index_b[0]));
            let s = dot(point_b - point_a, normal);
            if s < 0.0 {
                f.axis = -f.axis;
            }
        }
        f
    }

    fn find_min_separation(&self, t: f32) -> (usize, usize, f32) {
        let xf_a = self.sweep_a.get_transform(t);
        let xf_b = self.sweep_b.get_transform(t);
        match self.kind {
            SepType::Points => {
                let axis_a = mul_t_rv(xf_a.q, self.axis);
                let axis_b = mul_t_rv(xf_b.q, -self.axis);
                let ia = self.proxy_a.get_support(axis_a);
                let ib = self.proxy_b.get_support(axis_b);
                let point_a = mul_xv(&xf_a, self.proxy_a.get_vertex(ia));
                let point_b = mul_xv(&xf_b, self.proxy_b.get_vertex(ib));
                (ia, ib, dot(point_b - point_a, self.axis))
            }
            SepType::FaceA => {
                let normal = mul_rv(xf_a.q, self.axis);
                let point_a = mul_xv(&xf_a, self.local_point);
                let axis_b = mul_t_rv(xf_b.q, -normal);
                let ib = self.proxy_b.get_support(axis_b);
                let point_b = mul_xv(&xf_b, self.proxy_b.get_vertex(ib));
                (usize::MAX, ib, dot(point_b - point_a, normal))
            }
            SepType::FaceB => {
                let normal = mul_rv(xf_b.q, self.axis);
                let point_b = mul_xv(&xf_b, self.local_point);
                let axis_a = mul_t_rv(xf_a.q, -normal);
                let ia = self.proxy_a.get_support(axis_a);
                let point_a = mul_xv(&xf_a, self.proxy_a.get_vertex(ia));
                (ia, usize::MAX, dot(point_a - point_b, normal))
            }
        }
    }

    fn evaluate(&self, index_a: usize, index_b: usize, t: f32) -> f32 {
        let xf_a = self.sweep_a.get_transform(t);
        let xf_b = self.sweep_b.get_transform(t);
        match self.kind {
            SepType::Points => {
                let point_a = mul_xv(&xf_a, self.proxy_a.get_vertex(index_a));
                let point_b = mul_xv(&xf_b, self.proxy_b.get_vertex(index_b));
                dot(point_b - point_a, self.axis)
            }
            SepType::FaceA => {
                let normal = mul_rv(xf_a.q, self.axis);
                let point_a = mul_xv(&xf_a, self.local_point);
                let point_b = mul_xv(&xf_b, self.proxy_b.get_vertex(index_b));
                dot(point_b - point_a, normal)
            }
            SepType::FaceB => {
                let normal = mul_rv(xf_b.q, self.axis);
                let point_b = mul_xv(&xf_b, self.local_point);
                let point_a = mul_xv(&xf_a, self.proxy_a.get_vertex(index_a));
                dot(point_a - point_b, normal)
            }
        }
    }
}

/// b2TimeOfImpact.
pub fn time_of_impact(input: &ToiInput) -> ToiOutput {
    let mut output = ToiOutput {
        state: ToiState::Unknown,
        t: input.t_max,
    };
    let proxy_a = &input.proxy_a;
    let proxy_b = &input.proxy_b;
    let mut sweep_a = input.sweep_a;
    let mut sweep_b = input.sweep_b;
    sweep_a.normalize();
    sweep_b.normalize();
    let t_max = input.t_max;
    let total_radius = proxy_a.radius + proxy_b.radius;
    let target = max(LINEAR_SLOP, total_radius - 3.0 * LINEAR_SLOP);
    let tolerance = 0.25 * LINEAR_SLOP;
    let mut t1 = 0.0;
    const K_MAX_ITERATIONS: i32 = 20;
    let mut iter = 0;
    let mut cache = SimplexCache::default();
    let mut distance_input = DistanceInput {
        proxy_a: input.proxy_a,
        proxy_b: input.proxy_b,
        transform_a: Default::default(),
        transform_b: Default::default(),
        use_radii: false,
    };
    loop {
        distance_input.transform_a = sweep_a.get_transform(t1);
        distance_input.transform_b = sweep_b.get_transform(t1);
        let distance_output = distance(&mut cache, &distance_input);
        if distance_output.distance <= 0.0 {
            output.state = ToiState::Overlapped;
            output.t = 0.0;
            break;
        }
        if distance_output.distance < target + tolerance {
            output.state = ToiState::Touching;
            output.t = t1;
            break;
        }
        let fcn = SeparationFunction::initialize(&cache, proxy_a, sweep_a, proxy_b, sweep_b, t1);
        let mut done = false;
        let mut t2 = t_max;
        let mut push_back_iter = 0;
        loop {
            let (index_a, index_b, mut s2) = fcn.find_min_separation(t2);
            if s2 > target + tolerance {
                output.state = ToiState::Separated;
                output.t = t_max;
                done = true;
                break;
            }
            if s2 > target - tolerance {
                t1 = t2;
                break;
            }
            let mut s1 = fcn.evaluate(index_a, index_b, t1);
            if s1 < target - tolerance {
                output.state = ToiState::Failed;
                output.t = t1;
                done = true;
                break;
            }
            if s1 <= target + tolerance {
                output.state = ToiState::Touching;
                output.t = t1;
                done = true;
                break;
            }
            let mut root_iter_count = 0;
            let (mut a1, mut a2) = (t1, t2);
            loop {
                let t = if root_iter_count & 1 == 1 {
                    a1 + (target - s1) * (a2 - a1) / (s2 - s1)
                } else {
                    0.5 * (a1 + a2)
                };
                root_iter_count += 1;
                let s = fcn.evaluate(index_a, index_b, t);
                if (s - target).abs() < tolerance {
                    t2 = t;
                    break;
                }
                if s > target {
                    a1 = t;
                    s1 = s;
                } else {
                    a2 = t;
                    s2 = s;
                }
                if root_iter_count == 50 {
                    break;
                }
            }
            push_back_iter += 1;
            if push_back_iter == MAX_POLYGON_VERTICES {
                break;
            }
        }
        iter += 1;
        if done {
            break;
        }
        if iter == K_MAX_ITERATIONS {
            output.state = ToiState::Failed;
            output.t = t1;
            break;
        }
    }
    output
}
