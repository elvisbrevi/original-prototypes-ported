// b2Collision, b2CollideCircle y b2CollidePolygon: AABB, manifolds y la colisión entre círculos
// y polígonos.

use crate::math::{
    Transform, Vec2, cross_vs, distance_squared, dot, max_v, min_v, mul_rv, mul_t_rv, mul_t_xv,
    mul_t_xx, mul_xv, vec2,
};
use crate::settings::{EPSILON, LINEAR_SLOP, MAX_FLOAT, MAX_MANIFOLD_POINTS};
use crate::shapes::{CircleShape, PolygonShape};

pub const E_VERTEX: u8 = 0;
pub const E_FACE: u8 = 1;

/// b2ContactFeature / b2ContactID.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContactId {
    pub index_a: u8,
    pub index_b: u8,
    pub type_a: u8,
    pub type_b: u8,
}

impl ContactId {
    pub fn key(self) -> u32 {
        u32::from_le_bytes([self.index_a, self.index_b, self.type_a, self.type_b])
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ManifoldPoint {
    pub local_point: Vec2,
    pub normal_impulse: f32,
    pub tangent_impulse: f32,
    pub id: ContactId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ManifoldType {
    #[default]
    Circles,
    FaceA,
    FaceB,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Manifold {
    pub points: [ManifoldPoint; MAX_MANIFOLD_POINTS],
    pub local_normal: Vec2,
    pub local_point: Vec2,
    pub kind: ManifoldType,
    pub point_count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldManifold {
    pub normal: Vec2,
    pub points: [Vec2; MAX_MANIFOLD_POINTS],
    pub separations: [f32; MAX_MANIFOLD_POINTS],
}

impl WorldManifold {
    pub fn initialize(
        &mut self,
        manifold: &Manifold,
        xf_a: &Transform,
        radius_a: f32,
        xf_b: &Transform,
        radius_b: f32,
    ) {
        if manifold.point_count == 0 {
            return;
        }
        match manifold.kind {
            ManifoldType::Circles => {
                self.normal = vec2(1.0, 0.0);
                let point_a = mul_xv(xf_a, manifold.local_point);
                let point_b = mul_xv(xf_b, manifold.points[0].local_point);
                if distance_squared(point_a, point_b) > EPSILON * EPSILON {
                    self.normal = point_b - point_a;
                    self.normal.normalize();
                }
                let c_a = point_a + radius_a * self.normal;
                let c_b = point_b - radius_b * self.normal;
                self.points[0] = 0.5 * (c_a + c_b);
                self.separations[0] = dot(c_b - c_a, self.normal);
            }
            ManifoldType::FaceA => {
                self.normal = mul_rv(xf_a.q, manifold.local_normal);
                let plane_point = mul_xv(xf_a, manifold.local_point);
                for i in 0..manifold.point_count {
                    let clip_point = mul_xv(xf_b, manifold.points[i].local_point);
                    let c_a = clip_point
                        + (radius_a - dot(clip_point - plane_point, self.normal)) * self.normal;
                    let c_b = clip_point - radius_b * self.normal;
                    self.points[i] = 0.5 * (c_a + c_b);
                    self.separations[i] = dot(c_b - c_a, self.normal);
                }
            }
            ManifoldType::FaceB => {
                self.normal = mul_rv(xf_b.q, manifold.local_normal);
                let plane_point = mul_xv(xf_b, manifold.local_point);
                for i in 0..manifold.point_count {
                    let clip_point = mul_xv(xf_a, manifold.points[i].local_point);
                    let c_b = clip_point
                        + (radius_b - dot(clip_point - plane_point, self.normal)) * self.normal;
                    let c_a = clip_point - radius_a * self.normal;
                    self.points[i] = 0.5 * (c_a + c_b);
                    self.separations[i] = dot(c_a - c_b, self.normal);
                }
                self.normal = -self.normal;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClipVertex {
    pub v: Vec2,
    pub id: ContactId,
}

/// b2ClipSegmentToLine (Sutherland-Hodgman).
pub fn clip_segment_to_line(
    v_out: &mut [ClipVertex; 2],
    v_in: &[ClipVertex; 2],
    normal: Vec2,
    offset: f32,
    vertex_index_a: usize,
) -> usize {
    let mut num_out = 0;
    let distance0 = dot(normal, v_in[0].v) - offset;
    let distance1 = dot(normal, v_in[1].v) - offset;
    if distance0 <= 0.0 {
        v_out[num_out] = v_in[0];
        num_out += 1;
    }
    if distance1 <= 0.0 {
        v_out[num_out] = v_in[1];
        num_out += 1;
    }
    if distance0 * distance1 < 0.0 {
        let interp = distance0 / (distance0 - distance1);
        v_out[num_out].v = v_in[0].v + interp * (v_in[1].v - v_in[0].v);
        v_out[num_out].id = ContactId {
            index_a: vertex_index_a as u8,
            index_b: v_in[0].id.index_b,
            type_a: E_VERTEX,
            type_b: E_FACE,
        };
        num_out += 1;
    }
    num_out
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub lower_bound: Vec2,
    pub upper_bound: Vec2,
}

impl Aabb {
    pub fn get_perimeter(&self) -> f32 {
        let wx = self.upper_bound.x - self.lower_bound.x;
        let wy = self.upper_bound.y - self.lower_bound.y;
        2.0 * (wx + wy)
    }

    pub fn combine(a: &Aabb, b: &Aabb) -> Aabb {
        Aabb {
            lower_bound: min_v(a.lower_bound, b.lower_bound),
            upper_bound: max_v(a.upper_bound, b.upper_bound),
        }
    }

    pub fn contains(&self, aabb: &Aabb) -> bool {
        self.lower_bound.x <= aabb.lower_bound.x
            && self.lower_bound.y <= aabb.lower_bound.y
            && aabb.upper_bound.x <= self.upper_bound.x
            && aabb.upper_bound.y <= self.upper_bound.y
    }
}

pub fn test_overlap_aabb(a: &Aabb, b: &Aabb) -> bool {
    let d1 = b.lower_bound - a.upper_bound;
    let d2 = a.lower_bound - b.upper_bound;
    if d1.x > 0.0 || d1.y > 0.0 {
        return false;
    }
    if d2.x > 0.0 || d2.y > 0.0 {
        return false;
    }
    true
}

pub fn collide_circles(
    manifold: &mut Manifold,
    circle_a: &CircleShape,
    xf_a: &Transform,
    circle_b: &CircleShape,
    xf_b: &Transform,
) {
    manifold.point_count = 0;
    let p_a = mul_xv(xf_a, circle_a.p);
    let p_b = mul_xv(xf_b, circle_b.p);
    let d = p_b - p_a;
    let dist_sqr = dot(d, d);
    let radius = circle_a.radius + circle_b.radius;
    if dist_sqr > radius * radius {
        return;
    }
    manifold.kind = ManifoldType::Circles;
    manifold.local_point = circle_a.p;
    manifold.local_normal = Vec2::ZERO;
    manifold.point_count = 1;
    manifold.points[0].local_point = circle_b.p;
    manifold.points[0].id = ContactId::default();
}

pub fn collide_polygon_and_circle(
    manifold: &mut Manifold,
    polygon_a: &PolygonShape,
    xf_a: &Transform,
    circle_b: &CircleShape,
    xf_b: &Transform,
) {
    manifold.point_count = 0;
    let c = mul_xv(xf_b, circle_b.p);
    let c_local = mul_t_xv(xf_a, c);
    let mut normal_index = 0;
    let mut separation = -MAX_FLOAT;
    let radius = polygon_a.radius + circle_b.radius;
    let vertex_count = polygon_a.count;
    let vertices = &polygon_a.vertices;
    let normals = &polygon_a.normals;
    for i in 0..vertex_count {
        let s = dot(normals[i], c_local - vertices[i]);
        if s > radius {
            return;
        }
        if s > separation {
            separation = s;
            normal_index = i;
        }
    }
    let vert_index1 = normal_index;
    let vert_index2 = if vert_index1 + 1 < vertex_count {
        vert_index1 + 1
    } else {
        0
    };
    let v1 = vertices[vert_index1];
    let v2 = vertices[vert_index2];
    let un_punto = |m: &mut Manifold, normal: Vec2, point: Vec2| {
        m.point_count = 1;
        m.kind = ManifoldType::FaceA;
        m.local_normal = normal;
        m.local_point = point;
        m.points[0].local_point = circle_b.p;
        m.points[0].id = ContactId::default();
    };
    if separation < EPSILON {
        un_punto(manifold, normals[normal_index], 0.5 * (v1 + v2));
        return;
    }
    let u1 = dot(c_local - v1, v2 - v1);
    let u2 = dot(c_local - v2, v1 - v2);
    if u1 <= 0.0 {
        if distance_squared(c_local, v1) > radius * radius {
            return;
        }
        let mut n = c_local - v1;
        n.normalize();
        un_punto(manifold, n, v1);
    } else if u2 <= 0.0 {
        if distance_squared(c_local, v2) > radius * radius {
            return;
        }
        let mut n = c_local - v2;
        n.normalize();
        un_punto(manifold, n, v2);
    } else {
        let face_center = 0.5 * (v1 + v2);
        let separation = dot(c_local - face_center, normals[vert_index1]);
        if separation > radius {
            return;
        }
        un_punto(manifold, normals[vert_index1], face_center);
    }
}

/// b2FindMaxSeparation: la arista de poly1 que más separa a poly2.
fn find_max_separation(
    poly1: &PolygonShape,
    xf1: &Transform,
    poly2: &PolygonShape,
    xf2: &Transform,
) -> (usize, f32) {
    let xf = mul_t_xx(xf2, xf1);
    let mut best_index = 0;
    let mut max_separation = -MAX_FLOAT;
    for i in 0..poly1.count {
        let n = mul_rv(xf.q, poly1.normals[i]);
        let v1 = mul_xv(&xf, poly1.vertices[i]);
        let mut si = MAX_FLOAT;
        for j in 0..poly2.count {
            let sij = dot(n, poly2.vertices[j] - v1);
            if sij < si {
                si = sij;
            }
        }
        if si > max_separation {
            max_separation = si;
            best_index = i;
        }
    }
    (best_index, max_separation)
}

fn find_incident_edge(
    poly1: &PolygonShape,
    xf1: &Transform,
    edge1: usize,
    poly2: &PolygonShape,
    xf2: &Transform,
) -> [ClipVertex; 2] {
    let normal1 = mul_t_rv(xf2.q, mul_rv(xf1.q, poly1.normals[edge1]));
    let mut index = 0;
    let mut min_dot = MAX_FLOAT;
    for i in 0..poly2.count {
        let d = dot(normal1, poly2.normals[i]);
        if d < min_dot {
            min_dot = d;
            index = i;
        }
    }
    let i1 = index;
    let i2 = if i1 + 1 < poly2.count { i1 + 1 } else { 0 };
    let punto = |i: usize| ClipVertex {
        v: mul_xv(xf2, poly2.vertices[i]),
        id: ContactId {
            index_a: edge1 as u8,
            index_b: i as u8,
            type_a: E_FACE,
            type_b: E_VERTEX,
        },
    };
    [punto(i1), punto(i2)]
}

/// b2CollidePolygons: eje de separación (SAT) y recorte de la arista incidente.
pub fn collide_polygons(
    manifold: &mut Manifold,
    poly_a: &PolygonShape,
    xf_a: &Transform,
    poly_b: &PolygonShape,
    xf_b: &Transform,
) {
    manifold.point_count = 0;
    let total_radius = poly_a.radius + poly_b.radius;
    let (edge_a, separation_a) = find_max_separation(poly_a, xf_a, poly_b, xf_b);
    if separation_a > total_radius {
        return;
    }
    let (edge_b, separation_b) = find_max_separation(poly_b, xf_b, poly_a, xf_a);
    if separation_b > total_radius {
        return;
    }
    let k_tol = 0.1 * LINEAR_SLOP;
    let (poly1, poly2, xf1, xf2, edge1, flip) = if separation_b > separation_a + k_tol {
        manifold.kind = ManifoldType::FaceB;
        (poly_b, poly_a, xf_b, xf_a, edge_b, true)
    } else {
        manifold.kind = ManifoldType::FaceA;
        (poly_a, poly_b, xf_a, xf_b, edge_a, false)
    };
    let incident_edge = find_incident_edge(poly1, xf1, edge1, poly2, xf2);
    let count1 = poly1.count;
    let iv1 = edge1;
    let iv2 = if edge1 + 1 < count1 { edge1 + 1 } else { 0 };
    let mut v11 = poly1.vertices[iv1];
    let mut v12 = poly1.vertices[iv2];
    let mut local_tangent = v12 - v11;
    local_tangent.normalize();
    let local_normal = cross_vs(local_tangent, 1.0);
    let plane_point = 0.5 * (v11 + v12);
    let tangent = mul_rv(xf1.q, local_tangent);
    let normal = cross_vs(tangent, 1.0);
    v11 = mul_xv(xf1, v11);
    v12 = mul_xv(xf1, v12);
    let front_offset = dot(normal, v11);
    let side_offset1 = -dot(tangent, v11) + total_radius;
    let side_offset2 = dot(tangent, v12) + total_radius;
    let mut clip_points1 = [ClipVertex::default(); 2];
    let mut clip_points2 = [ClipVertex::default(); 2];
    let np = clip_segment_to_line(
        &mut clip_points1,
        &incident_edge,
        -tangent,
        side_offset1,
        iv1,
    );
    if np < 2 {
        return;
    }
    let np = clip_segment_to_line(&mut clip_points2, &clip_points1, tangent, side_offset2, iv2);
    if np < 2 {
        return;
    }
    manifold.local_normal = local_normal;
    manifold.local_point = plane_point;
    let mut point_count = 0;
    for cp2 in clip_points2.iter().take(MAX_MANIFOLD_POINTS) {
        let separation = dot(normal, cp2.v) - front_offset;
        if separation <= total_radius {
            let cp = &mut manifold.points[point_count];
            cp.local_point = mul_t_xv(xf2, cp2.v);
            cp.id = cp2.id;
            if flip {
                let cf = cp.id;
                cp.id = ContactId {
                    index_a: cf.index_b,
                    index_b: cf.index_a,
                    type_a: cf.type_b,
                    type_b: cf.type_a,
                };
            }
            point_count += 1;
        }
    }
    manifold.point_count = point_count;
}
