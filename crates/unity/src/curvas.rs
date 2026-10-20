// AnimationCurve (Hermite entre claves, constante fuera de ellas), MinMaxCurve, Gradient y
// MinMaxGradient, como los evalúan los módulos de partículas de Unity.

use macroquad::prelude::Color;

use crate::datos::{Gradiente, MinMaxCurva, MinMaxGradiente};

/// AnimationCurve.Evaluate con claves (tiempo, valor, tangente de entrada, de salida). Antes de
/// la primera y después de la última clave vale lo que ellas (PreInfinity/PostInfinity = Clamp,
/// los de todas las curvas de estos juegos). Una tangente infinita es un escalón.
pub fn evaluar(claves: &[[f32; 4]], t: f32) -> f32 {
    let (Some(primera), Some(ultima)) = (claves.first(), claves.last()) else {
        return 0.0;
    };
    if t <= primera[0] {
        return primera[1];
    }
    if t >= ultima[0] {
        return ultima[1];
    }
    let i = claves.iter().rposition(|k| k[0] <= t).unwrap_or(0);
    let (a, b) = (claves[i], claves[i + 1]);
    let dt = b[0] - a[0];
    if dt <= 0.0 {
        return b[1];
    }
    if !a[3].is_finite() || !b[2].is_finite() {
        return a[1];
    }
    let s = (t - a[0]) / dt;
    let (s2, s3) = (s * s, s * s * s);
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    h00 * a[1] + h10 * dt * a[3] + h01 * b[1] + h11 * dt * b[2]
}

impl MinMaxCurva {
    /// El valor en el tiempo normalizado `t`, con `azar` (0 a 1) fijo para cada partícula en los
    /// modos aleatorios.
    pub fn evaluar(&self, t: f32, azar: f32) -> f32 {
        let v = match self.modo {
            0 => 1.0,
            1 => evaluar(&self.max, t),
            2 => {
                let (a, b) = (evaluar(&self.min, t), evaluar(&self.max, t));
                a + (b - a) * azar
            }
            _ => {
                let a = self.min.first().map_or(0.0, |k| k[1]);
                let b = self.max.first().map_or(0.0, |k| k[1]);
                a + (b - a) * azar
            }
        };
        self.escalar * v
    }
}

fn mezclar(a: Color, b: Color, f: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * f,
        a.g + (b.g - a.g) * f,
        a.b + (b.b - a.b) * f,
        a.a + (b.a - a.a) * f,
    )
}

fn tramo<const N: usize>(
    claves: &[[f32; N]],
    t: f32,
    valor: impl Fn(&[f32; N]) -> [f32; 3],
) -> [f32; 3] {
    let (Some(primera), Some(ultima)) = (claves.first(), claves.last()) else {
        return [1.0; 3];
    };
    if t <= primera[0] {
        return valor(primera);
    }
    if t >= ultima[0] {
        return valor(ultima);
    }
    let i = claves.iter().rposition(|k| k[0] <= t).unwrap_or(0);
    let (a, b) = (&claves[i], &claves[i + 1]);
    let f = if b[0] > a[0] {
        (t - a[0]) / (b[0] - a[0])
    } else {
        1.0
    };
    let (va, vb) = (valor(a), valor(b));
    [0, 1, 2].map(|c| va[c] + (vb[c] - va[c]) * f)
}

impl Gradiente {
    /// Gradient.Evaluate: el color y el alfa se interpolan por separado entre sus claves.
    pub fn evaluar(&self, t: f32) -> Color {
        let c = tramo(&self.colores, t, |k| [k[1], k[2], k[3]]);
        let a = tramo(&self.alfas, t, |k| [k[1], 0.0, 0.0]);
        Color::new(c[0], c[1], c[2], a[0])
    }
}

impl MinMaxGradiente {
    pub fn evaluar(&self, t: f32, azar: f32) -> Color {
        let c = |v: [f32; 4]| Color::new(v[0], v[1], v[2], v[3]);
        match self.modo {
            0 => c(self.max_color),
            1 => self.max.evaluar(t),
            2 => mezclar(c(self.min_color), c(self.max_color), azar),
            _ => mezclar(self.min.evaluar(t), self.max.evaluar(t), azar),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datos::Gradiente;

    #[test]
    fn hermite_y_extremos() {
        // La curva de tamaño de dobleSalto: de 0,502 a 1 con tangentes 0,498 (una recta).
        let c = [[0.0, 0.502, 0.498, 0.498], [1.0, 1.0, 0.498, 0.498]];
        assert!((evaluar(&c, 0.5) - 0.751).abs() < 1e-5);
        assert_eq!(evaluar(&c, -1.0), 0.502);
        assert_eq!(evaluar(&c, 2.0), 1.0);
        // Con tangentes planas es una S: en la mitad, el promedio.
        let s = [[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 0.0, 0.0]];
        assert!((evaluar(&s, 0.5) - 0.5).abs() < 1e-6);
        assert!(evaluar(&s, 0.25) < 0.25);
    }

    #[test]
    fn min_max_entre_constantes() {
        // startLifetime de dobleSalto: 0,2 por un valor entre 0,5 y 1.
        let c = MinMaxCurva {
            modo: 3,
            escalar: 0.2,
            max: vec![[0.0, 1.0, 0.0, 0.0]],
            min: vec![[0.0, 0.5, 0.0, 0.0]],
        };
        assert!((c.evaluar(0.3, 0.0) - 0.1).abs() < 1e-6);
        assert!((c.evaluar(0.3, 1.0) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn gradiente_por_separado() {
        let g = Gradiente {
            colores: vec![[0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]],
            alfas: vec![[0.0, 1.0], [0.5, 0.0]],
        };
        let c = g.evaluar(0.25);
        assert!((c.r - 0.75).abs() < 1e-6 && (c.b - 0.25).abs() < 1e-6);
        assert!((c.a - 0.5).abs() < 1e-6);
        assert_eq!(g.evaluar(0.9).a, 0.0);
    }
}
