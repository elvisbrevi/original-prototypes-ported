// Geometría de colisión del runtime (common_prelude.js): rectángulos, cuadriláteros y polígonos
// (cr.rect, cr.quad, cr.CollisionPoly_), cr.segments_intersect y las funciones de ángulos. Todo
// en f64, como los números de JavaScript.

use std::f64::consts::PI;

use macroquad::math::DVec2;

/// cr.rect. Hasta normalize() puede tener left > right (ancho negativo, Sprite espejado).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Caja {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Caja {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn offset(&mut self, dx: f64, dy: f64) {
        self.left += dx;
        self.top += dy;
        self.right += dx;
        self.bottom += dy;
    }

    pub fn normalize(&mut self) {
        if self.left > self.right {
            std::mem::swap(&mut self.left, &mut self.right);
        }
        if self.top > self.bottom {
            std::mem::swap(&mut self.top, &mut self.bottom);
        }
    }

    pub fn ancho(&self) -> f64 {
        self.right - self.left
    }

    pub fn alto(&self) -> f64 {
        self.bottom - self.top
    }

    /// intersects_rect: los bordes que se tocan cuentan.
    pub fn se_toca(&self, o: &Caja) -> bool {
        !(o.right < self.left || o.bottom < self.top || o.left > self.right || o.top > self.bottom)
    }

    /// contains_pt.
    pub fn contiene(&self, x: f64, y: f64) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

/// cr.quad: las esquinas en el orden tl, tr, br, bl.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cuadrilatero {
    pub x: [f64; 4],
    pub y: [f64; 4],
}

impl Cuadrilatero {
    /// set_from_rect.
    pub fn desde_caja(c: &Caja) -> Self {
        Self {
            x: [c.left, c.right, c.right, c.left],
            y: [c.top, c.top, c.bottom, c.bottom],
        }
    }

    /// set_from_rotated_rect: la caja girada alrededor del origen.
    pub fn desde_caja_rotada(c: &Caja, a: f64) -> Self {
        if a == 0.0 {
            return Self::desde_caja(c);
        }
        let (sin_a, cos_a) = (a.sin(), a.cos());
        let (left_sin, top_sin, right_sin, bottom_sin) = (
            c.left * sin_a,
            c.top * sin_a,
            c.right * sin_a,
            c.bottom * sin_a,
        );
        let (left_cos, top_cos, right_cos, bottom_cos) = (
            c.left * cos_a,
            c.top * cos_a,
            c.right * cos_a,
            c.bottom * cos_a,
        );
        Self {
            x: [
                left_cos - top_sin,
                right_cos - top_sin,
                right_cos - bottom_sin,
                left_cos - bottom_sin,
            ],
            y: [
                top_cos + left_sin,
                top_cos + right_sin,
                bottom_cos + right_sin,
                bottom_cos + left_sin,
            ],
        }
    }

    pub fn offset(&mut self, dx: f64, dy: f64) {
        for i in 0..4 {
            self.x[i] += dx;
            self.y[i] += dy;
        }
    }

    /// bounding_box.
    pub fn caja(&self) -> Caja {
        let (min_x, max_x) = min_max4(self.x);
        let (min_y, max_y) = min_max4(self.y);
        Caja::new(min_x, min_y, max_x, max_y)
    }

    /// contains_pt: coordenadas baricéntricas en los triángulos (tl, tr, br) y (tl, bl, br).
    pub fn contiene(&self, x: f64, y: f64) -> bool {
        let (ox, oy) = (self.x[0], self.y[0]);
        let (v1x, v1y) = (self.x[2] - ox, self.y[2] - oy);
        let (v2x, v2y) = (x - ox, y - oy);
        let dot11 = v1x * v1x + v1y * v1y;
        let dot12 = v1x * v2x + v1y * v2y;
        let en_triangulo = |v0x: f64, v0y: f64| {
            let dot00 = v0x * v0x + v0y * v0y;
            let dot01 = v0x * v1x + v0y * v1y;
            let dot02 = v0x * v2x + v0y * v2y;
            let inv = 1.0 / (dot00 * dot11 - dot01 * dot01);
            let u = (dot11 * dot02 - dot01 * dot12) * inv;
            let v = (dot00 * dot12 - dot01 * dot02) * inv;
            u >= 0.0 && v > 0.0 && u + v < 1.0
        };
        en_triangulo(self.x[1] - ox, self.y[1] - oy) || en_triangulo(self.x[3] - ox, self.y[3] - oy)
    }

    fn medio(&self) -> (f64, f64) {
        (
            (self.x[0] + self.x[1] + self.x[2] + self.x[3]) / 4.0,
            (self.y[0] + self.y[1] + self.y[2] + self.y[3]) / 4.0,
        )
    }

    /// intersects_quad: el centro de uno dentro del otro, o algún par de lados que se cortan.
    pub fn se_corta(&self, o: &Cuadrilatero) -> bool {
        let (mx, my) = o.medio();
        if self.contiene(mx, my) {
            return true;
        }
        let (mx, my) = self.medio();
        if o.contiene(mx, my) {
            return true;
        }
        for i in 0..4 {
            let i2 = (i + 1) % 4;
            for j in 0..4 {
                let j2 = (j + 1) % 4;
                if segmentos_se_cortan(
                    self.x[i], self.y[i], self.x[i2], self.y[i2], o.x[j], o.y[j], o.x[j2], o.y[j2],
                ) {
                    return true;
                }
            }
        }
        false
    }
}

fn min_max4(v: [f64; 4]) -> (f64, f64) {
    let mut min = v[0];
    let mut max = v[0];
    for &x in &v[1..] {
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }
    (min, max)
}

/// cr.segments_intersect.
#[allow(clippy::too_many_arguments)]
pub fn segmentos_se_cortan(
    a1x: f64,
    a1y: f64,
    a2x: f64,
    a2y: f64,
    b1x: f64,
    b1y: f64,
    b2x: f64,
    b2y: f64,
) -> bool {
    let (min_ax, max_ax) = if a1x < a2x { (a1x, a2x) } else { (a2x, a1x) };
    let (min_bx, max_bx) = if b1x < b2x { (b1x, b2x) } else { (b2x, b1x) };
    if max_ax < min_bx || min_ax > max_bx {
        return false;
    }
    let (min_ay, max_ay) = if a1y < a2y { (a1y, a2y) } else { (a2y, a1y) };
    let (min_by, max_by) = if b1y < b2y { (b1y, b2y) } else { (b2y, b1y) };
    if max_ay < min_by || min_ay > max_by {
        return false;
    }
    let dpx = b1x - a1x + b2x - a2x;
    let dpy = b1y - a1y + b2y - a2y;
    let (qax, qay) = (a2x - a1x, a2y - a1y);
    let (qbx, qby) = (b2x - b1x, b2y - b1y);
    let d = (qay * qbx - qby * qax).abs();
    if (qbx * dpy - qby * dpx).abs() > d {
        return false;
    }
    (qax * dpy - qay * dpx).abs() <= d
}

/// CollisionPoly_ ya transformado (cache_poly o set_from_quad): los puntos relativos a la
/// posición de la instancia y su caja.
#[derive(Clone, Debug)]
pub struct Poligono {
    pub puntos: Vec<DVec2>,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Poligono {
    fn con_puntos(puntos: Vec<DVec2>) -> Self {
        let mut p = Self {
            min_x: puntos[0].x,
            min_y: puntos[0].y,
            max_x: puntos[0].x,
            max_y: puntos[0].y,
            puntos,
        };
        for q in &p.puntos[1..] {
            p.min_x = p.min_x.min(q.x);
            p.max_x = p.max_x.max(q.x);
            p.min_y = p.min_y.min(q.y);
            p.max_y = p.max_y.max(q.y);
        }
        p
    }

    /// cache_poly: el polígono del cuadro (en fracciones del tamaño, relativo al origen) escalado
    /// por el tamaño de la instancia (negativo si está espejada) y girado.
    pub fn desde_fracciones(fracciones: &[DVec2], ancho: f64, alto: f64, angulo: f64) -> Self {
        let (s, c) = if angulo != 0.0 {
            (angulo.sin(), angulo.cos())
        } else {
            (0.0, 1.0)
        };
        Self::con_puntos(
            fracciones
                .iter()
                .map(|p| {
                    let (x, y) = (p.x * ancho, p.y * alto);
                    DVec2::new(x * c - y * s, y * c + x * s)
                })
                .collect(),
        )
    }

    /// set_from_quad: las esquinas del cuadrilátero menos la posición de la instancia.
    pub fn desde_cuadrilatero(q: &Cuadrilatero, x: f64, y: f64) -> Self {
        Self::con_puntos((0..4).map(|i| DVec2::new(q.x[i] - x, q.y[i] - y)).collect())
    }

    /// contains_pt: dos rayos desde fuera de la caja; dentro si alguno cruza un número impar de
    /// lados.
    pub fn contiene(&self, x: f64, y: f64) -> bool {
        let p = &self.puntos;
        if x == p[0].x && y == p[0].y {
            return true;
        }
        let (r1x, r1y) = (self.min_x - 110.0, self.min_y - 101.0);
        let (r2x, r2y) = (self.max_x + 131.0, self.max_y + 120.0);
        let (mut c1, mut c2) = (0, 0);
        for i in 0..p.len() {
            let j = (i + 1) % p.len();
            if segmentos_se_cortan(r1x, r1y, x, y, p[i].x, p[i].y, p[j].x, p[j].y) {
                c1 += 1;
            }
            if segmentos_se_cortan(r2x, r2y, x, y, p[i].x, p[i].y, p[j].x, p[j].y) {
                c2 += 1;
            }
        }
        c1 % 2 == 1 || c2 % 2 == 1
    }

    /// intersects_poly: `otro` está desplazado (dx, dy) respecto de este.
    pub fn se_corta(&self, otro: &Poligono, dx: f64, dy: f64) -> bool {
        let (a, b) = (&self.puntos, &otro.puntos);
        if self.contiene(b[0].x + dx, b[0].y + dy) || otro.contiene(a[0].x - dx, a[0].y - dy) {
            return true;
        }
        for i in 0..a.len() {
            let i2 = (i + 1) % a.len();
            for j in 0..b.len() {
                let j2 = (j + 1) % b.len();
                if segmentos_se_cortan(
                    a[i].x,
                    a[i].y,
                    a[i2].x,
                    a[i2].y,
                    b[j].x + dx,
                    b[j].y + dy,
                    b[j2].x + dx,
                    b[j2].y + dy,
                ) {
                    return true;
                }
            }
        }
        false
    }
}

/// cr.to_radians.
pub fn a_radianes(grados: f64) -> f64 {
    grados / (180.0 / PI)
}

/// cr.to_degrees.
pub fn a_grados(radianes: f64) -> f64 {
    180.0 / PI * radianes
}

/// cr.clamp_angle_degrees: en [0, 360).
pub fn grados_acotados(g: f64) -> f64 {
    let g = g % 360.0;
    if g < 0.0 { g + 360.0 } else { g }
}

/// cr.clamp_angle: en [0, 2π).
pub fn angulo_acotado(a: f64) -> f64 {
    let a = a % (2.0 * PI);
    if a < 0.0 { a + 2.0 * PI } else { a }
}

/// cr.to_clamped_degrees (la expresión Angle).
pub fn grados_de(a: f64) -> f64 {
    grados_acotados(a_grados(a))
}

/// cr.angleTo.
pub fn angulo_hacia(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (y2 - y1).atan2(x2 - x1)
}

/// cr.distanceTo.
pub fn distancia(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let (dx, dy) = (x2 - x1, y2 - y1);
    (dx * dx + dy * dy).sqrt()
}

/// cr.angleDiff: en [0, π].
pub fn diferencia_angular(a: f64, b: f64) -> f64 {
    if a == b {
        return 0.0;
    }
    let p = a.sin() * b.sin() + a.cos() * b.cos();
    if p >= 1.0 {
        0.0
    } else if p <= -1.0 {
        PI
    } else {
        p.acos()
    }
}

/// cr.angleRotate: gira `inicio` hacia `fin` como mucho `paso` radianes.
pub fn rotar_angulo(inicio: f64, fin: f64, paso: f64) -> f64 {
    let (ss, cs) = (inicio.sin(), inicio.cos());
    let (se, ce) = (fin.sin(), fin.cos());
    if (ss * se + cs * ce).acos() > paso {
        if cs * se - ss * ce > 0.0 {
            angulo_acotado(inicio + paso)
        } else {
            angulo_acotado(inicio - paso)
        }
    } else {
        angulo_acotado(fin)
    }
}

/// cr.angleClockwise: si `a` está en sentido horario respecto de `b`.
pub fn en_sentido_horario(a: f64, b: f64) -> bool {
    a.cos() * b.sin() - a.sin() * b.cos() <= 0.0
}

/// cr.round6dp, con el redondeo de Math.round (las mitades hacia +∞).
pub fn redondear6(x: f64) -> f64 {
    redondear(1e6 * x) / 1e6
}

/// Math.round.
pub fn redondear(x: f64) -> f64 {
    let piso = x.floor();
    if x - piso >= 0.5 { piso + 1.0 } else { piso }
}

/// cr.floor tal como está en el runtime: x|0 si x ≥ 0 y (x|0) − 1 si no, así que los enteros
/// negativos bajan uno más.
pub fn piso_c2(x: f64) -> f64 {
    if x >= 0.0 { x.trunc() } else { x.trunc() - 1.0 }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn redondeo_como_javascript() {
        assert_eq!(redondear(-0.5), 0.0);
        assert_eq!(redondear(2.5), 3.0);
        assert_eq!(redondear6(0.1 + 0.2), 0.3);
        assert_eq!(piso_c2(-3.0), -4.0);
        assert_eq!(piso_c2(-2.5), -3.0);
        assert_eq!(piso_c2(2.5), 2.0);
    }

    #[test]
    fn cuadrilatero_contiene() {
        let q = Cuadrilatero::desde_caja(&Caja::new(0.0, 0.0, 10.0, 10.0));
        assert!(q.contiene(5.0, 5.0));
        assert!(!q.contiene(11.0, 5.0));
        let r = Cuadrilatero::desde_caja_rotada(&Caja::new(-5.0, -5.0, 5.0, 5.0), PI / 4.0);
        assert!(r.contiene(0.0, 6.5));
        assert!(!r.contiene(5.0, 5.0));
    }

    #[test]
    fn angulos() {
        let a = rotar_angulo(0.0, PI / 2.0, 0.1);
        assert!((a - 0.1).abs() < 1e-12);
        assert!((rotar_angulo(0.0, -PI / 2.0, 0.1) - (2.0 * PI - 0.1)).abs() < 1e-12);
        assert!((grados_de(-PI / 2.0) - 270.0).abs() < 1e-9);
    }
}
