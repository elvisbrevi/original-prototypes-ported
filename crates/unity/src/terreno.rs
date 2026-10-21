// El terreno de Unity 3.x (Terrain + TerrainData): la malla del mapa de alturas en la zona que ve
// la cámara (con la resolución completa, como el LOD de Unity de cerca, con «pixel error» 1), las
// capas de textura mezcladas con el mapa de control y los detalles de pasto (billboards que se
// balancean con el viento). Sin árboles: el terreno del menú de Go Ball no tiene ninguno.
//
// Unity reparte el pasto de cada muestra del mapa de detalles al azar dentro de la muestra, con
// un tamaño y un color (entre el sano y el seco) que dependen de un ruido de Perlin; aquí el azar
// es otro (un generador propio con semilla por parche), así que las matas no quedan en los mismos
// lugares, pero sí en las mismas muestras y en la misma cantidad.

use macroquad::prelude::{Vec3, Vec4, vec2, vec3};

use crate::datos3d::{DatosTerreno, ParcheDetalle};

/// Un vértice del terreno o del pasto.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VerticeTerreno {
    pub posicion: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    /// En el pasto: el desplazamiento del billboard (x a lo ancho, y hacia arriba).
    pub tangente: [f32; 4],
}

pub struct Terreno {
    pub datos: DatosTerreno,
    /// Alturas de 0 a 1 (fila por fila en z).
    alturas: Vec<f32>,
    pub origen: Vec3,
}

/// Unity guarda las alturas como enteros de 0 a 32766.
const ALTURA_MAXIMA: f32 = 32766.0;

/// Un generador de números al azar pequeño y determinista (xorshift32).
struct Azar(u32);

impl Azar {
    fn siguiente(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// Ruido de Perlin 2D (el clásico, con la tabla de permutación de Ken Perlin), entre 0 y 1
/// aproximadamente, como Mathf.PerlinNoise.
fn perlin(x: f32, y: f32) -> f32 {
    const P: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
        69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
        252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
        168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
        211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
        216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
        164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
        126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
        213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
        253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
        192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
        138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];
    let p = |i: i32| P[(i & 255) as usize] as i32;
    let grad = |h: i32, x: f32, y: f32| {
        let u = if h & 1 == 0 { x } else { -x };
        let v = if h & 2 == 0 { y } else { -y };
        u + v
    };
    let suave = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (xf, yf) = (x - x.floor(), y - y.floor());
    let (u, v) = (suave(xf), suave(yf));
    let aa = p(p(xi) + yi);
    let ab = p(p(xi) + yi + 1);
    let ba = p(p(xi + 1) + yi);
    let bb = p(p(xi + 1) + yi + 1);
    let x1 = grad(aa, xf, yf) + u * (grad(ba, xf - 1.0, yf) - grad(aa, xf, yf));
    let x2 = grad(ab, xf, yf - 1.0) + u * (grad(bb, xf - 1.0, yf - 1.0) - grad(ab, xf, yf - 1.0));
    ((x1 + v * (x2 - x1)) + 1.0) / 2.0
}

impl Terreno {
    /// `alturas`: las muestras (0 a 255, que en este terreno es todo su rango) fila por fila en z.
    pub fn nuevo(datos: &DatosTerreno, alturas: Vec<u8>, origen: Vec3) -> Self {
        Self {
            datos: datos.clone(),
            alturas: alturas.iter().map(|&h| h as f32 / ALTURA_MAXIMA).collect(),
            origen,
        }
    }

    fn muestra(&self, i: i32, j: i32) -> f32 {
        let [w, h] = self.datos.alturas;
        let i = i.clamp(0, w as i32 - 1) as usize;
        let j = j.clamp(0, h as i32 - 1) as usize;
        self.alturas[j * w as usize + i] * self.datos.escala[1]
    }

    /// TerrainData.GetInterpolatedHeight en coordenadas del terreno (sin el origen).
    pub fn altura(&self, x: f32, z: f32) -> f32 {
        let fx = x / self.datos.escala[0];
        let fz = z / self.datos.escala[2];
        let (i, j) = (fx.floor() as i32, fz.floor() as i32);
        let (tx, tz) = (fx - i as f32, fz - j as f32);
        let a = self.muestra(i, j);
        let b = self.muestra(i + 1, j);
        let c = self.muestra(i, j + 1);
        let d = self.muestra(i + 1, j + 1);
        // Los dos triángulos de cada celda, como la malla.
        if tx + tz <= 1.0 {
            a + (b - a) * tx + (c - a) * tz
        } else {
            d + (c - d) * (1.0 - tx) + (b - d) * (1.0 - tz)
        }
    }

    fn normal(&self, i: i32, j: i32) -> Vec3 {
        let (sx, sz) = (self.datos.escala[0], self.datos.escala[2]);
        let dx = (self.muestra(i + 1, j) - self.muestra(i - 1, j)) / (2.0 * sx);
        let dz = (self.muestra(i, j + 1) - self.muestra(i, j - 1)) / (2.0 * sz);
        vec3(-dx, 1.0, -dz).normalize()
    }

    /// La malla del terreno en las muestras cuyo centro queda a menos de `radio` (en x y z) de
    /// `centro` (en el mundo): vértices e índices de triángulos.
    pub fn malla(&self, centro: Vec3, radio: f32) -> (Vec<VerticeTerreno>, Vec<u16>) {
        let [w, h] = self.datos.alturas;
        let (sx, sz) = (self.datos.escala[0], self.datos.escala[2]);
        let local = centro - self.origen;
        let rango = |c: f32, s: f32, n: u32| {
            let a = ((c - radio) / s).floor().max(0.0) as i32;
            let b = ((c + radio) / s).ceil().min(n as f32 - 1.0) as i32;
            (a, b.max(a))
        };
        let (i0, i1) = rango(local.x, sx, w);
        let (j0, j1) = rango(local.z, sz, h);
        let mut vertices = Vec::new();
        for j in j0..=j1 {
            for i in i0..=i1 {
                let p = vec3(i as f32 * sx, self.muestra(i, j), j as f32 * sz) + self.origen;
                vertices.push(VerticeTerreno {
                    posicion: p.to_array(),
                    normal: self.normal(i, j).to_array(),
                    uv: [i as f32 / (w - 1) as f32, j as f32 / (h - 1) as f32],
                    color: [1.0; 4],
                    tangente: [0.0; 4],
                });
            }
        }
        let ancho = (i1 - i0 + 1) as u16;
        let mut indices = Vec::new();
        for j in 0..(j1 - j0) as u16 {
            for i in 0..(i1 - i0) as u16 {
                let a = j * ancho + i;
                let (b, c, d) = (a + 1, a + ancho, a + ancho + 1);
                // El orden de Unity (vistas desde arriba, en sentido horario en su mano izquierda).
                indices.extend_from_slice(&[a, c, d, a, d, b]);
            }
        }
        (vertices, indices)
    }

    /// Los billboards de pasto de los parches con detalles a menos de `distancia` de `centro`:
    /// por cada capa (prototipo), sus vértices e índices.
    pub fn pasto(
        &self,
        parches: &[ParcheDetalle],
        centro: Vec3,
        distancia: f32,
        densidad: f32,
    ) -> Vec<(usize, Vec<VerticeTerreno>, Vec<u16>)> {
        let d = &self.datos.detalles;
        let tamano = vec2(
            self.datos.escala[0] * (self.datos.alturas[0] - 1) as f32,
            self.datos.escala[2] * (self.datos.alturas[1] - 1) as f32,
        );
        let celda = tamano / (d.parches * d.muestras) as f32;
        let parche = celda * d.muestras as f32;
        let local = centro - self.origen;
        let mut capas: Vec<(usize, Vec<VerticeTerreno>, Vec<u16>)> = (0..d.prototipos.len())
            .map(|i| (i, Vec::new(), Vec::new()))
            .collect();
        let n = d.muestras as usize;
        for p in parches {
            let esquina = vec2(p.x as f32, p.z as f32) * parche;
            let cerca = vec2(
                local.x.clamp(esquina.x, esquina.x + parche.x),
                local.z.clamp(esquina.y, esquina.y + parche.y),
            );
            if cerca.distance(vec2(local.x, local.z)) > distancia {
                continue;
            }
            let mut azar = Azar(1 + p.x * 7919 + p.z * 104_729);
            for (k, &capa) in p.capas.iter().enumerate() {
                let proto = &d.prototipos[capa as usize];
                let (_, vertices, indices) = &mut capas[capa as usize];
                for mz in 0..n {
                    for mx in 0..n {
                        let cantidad = p.cantidades[k * n * n + mz * n + mx] as f32 * densidad;
                        for _ in 0..cantidad.round() as usize {
                            let pos = esquina
                                + vec2(
                                    (mx as f32 + azar.siguiente()) * celda.x,
                                    (mz as f32 + azar.siguiente()) * celda.y,
                                );
                            let ruido = perlin(pos.x * proto.ruido, pos.y * proto.ruido);
                            let ancho = proto.ancho[0] + (proto.ancho[1] - proto.ancho[0]) * ruido;
                            let alto = proto.alto[0] + (proto.alto[1] - proto.alto[0]) * ruido;
                            let c = Vec4::from(proto.sano).lerp(Vec4::from(proto.seco), ruido);
                            let base = vec3(pos.x, self.altura(pos.x, pos.y), pos.y) + self.origen;
                            let i = vertices.len() as u16;
                            if i as usize + 4 > u16::MAX as usize {
                                continue;
                            }
                            for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                                vertices.push(VerticeTerreno {
                                    posicion: base.to_array(),
                                    normal: [0.0, 1.0, 0.0],
                                    uv: [u, v],
                                    color: c.to_array(),
                                    tangente: [(u - 0.5) * ancho, v * alto, 0.0, 0.0],
                                });
                            }
                            indices.extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
                        }
                    }
                }
            }
        }
        capas.retain(|(_, v, _)| !v.is_empty());
        capas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_entre_cero_y_uno() {
        for i in 0..100 {
            let v = perlin(i as f32 * 0.37, i as f32 * 0.11);
            assert!((-0.1..=1.1).contains(&v), "{v}");
        }
    }
}
