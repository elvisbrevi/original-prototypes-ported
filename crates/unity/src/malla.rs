// Las mallas de Unity 3.x tal como las escribe la herramienta de extracción (mallas/<clave>.bin):
// vértices con sus canales, los triángulos de cada submalla (Unity 3.4 los guardaba como tiras y
// la herramienta ya los separó) y los triángulos que usa el MeshCollider.

use macroquad::prelude::{Vec2, Vec3, Vec4, vec2, vec3, vec4};

pub struct Malla {
    pub posiciones: Vec<Vec3>,
    pub normales: Vec<Vec3>,
    pub uv: Vec<Vec2>,
    pub uv2: Vec<Vec2>,
    /// Color32 (rgba) por vértice; vacío si la malla no trae colores.
    pub colores: Vec<[u8; 4]>,
    pub tangentes: Vec<Vec4>,
    /// Índices de los triángulos de cada submalla.
    pub submallas: Vec<Vec<u16>>,
    /// Índices de los triángulos de colisión (m_CollisionTriangles).
    pub colision: Vec<u16>,
}

struct Lector<'a> {
    datos: &'a [u8],
    pos: usize,
}

impl Lector<'_> {
    fn u32(&mut self) -> u32 {
        let b = &self.datos[self.pos..self.pos + 4];
        self.pos += 4;
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }

    fn f32(&mut self) -> f32 {
        f32::from_bits(self.u32())
    }

    fn u16(&mut self) -> u16 {
        let b = &self.datos[self.pos..self.pos + 2];
        self.pos += 2;
        u16::from_le_bytes([b[0], b[1]])
    }
}

impl Malla {
    pub fn leer(datos: &[u8]) -> Self {
        assert_eq!(&datos[..8], b"MALLA3D\0", "malla");
        let mut l = Lector { datos, pos: 8 };
        let n = l.u32() as usize;
        let banderas = l.u32();
        let posiciones = (0..n).map(|_| vec3(l.f32(), l.f32(), l.f32())).collect();
        let normales = if banderas & 1 != 0 {
            (0..n).map(|_| vec3(l.f32(), l.f32(), l.f32())).collect()
        } else {
            Vec::new()
        };
        let uv = if banderas & 2 != 0 {
            (0..n).map(|_| vec2(l.f32(), l.f32())).collect()
        } else {
            Vec::new()
        };
        let uv2 = if banderas & 16 != 0 {
            (0..n).map(|_| vec2(l.f32(), l.f32())).collect()
        } else {
            Vec::new()
        };
        let colores = if banderas & 4 != 0 {
            (0..n).map(|_| l.u32().to_le_bytes()).collect()
        } else {
            Vec::new()
        };
        let tangentes = if banderas & 8 != 0 {
            (0..n)
                .map(|_| vec4(l.f32(), l.f32(), l.f32(), l.f32()))
                .collect()
        } else {
            Vec::new()
        };
        let subs = l.u32() as usize;
        let submallas = (0..subs)
            .map(|_| {
                let k = l.u32() as usize;
                (0..k).map(|_| l.u16()).collect()
            })
            .collect();
        let k = l.u32() as usize;
        let colision = (0..k).map(|_| l.u16()).collect();
        Self {
            posiciones,
            normales,
            uv,
            uv2,
            colores,
            tangentes,
            submallas,
            colision,
        }
    }

    /// Los triángulos de colisión como triples de posiciones locales.
    pub fn triangulos_colision(&self) -> Vec<[Vec3; 3]> {
        self.colision
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| {
                [
                    self.posiciones[t[0] as usize],
                    self.posiciones[t[1] as usize],
                    self.posiciones[t[2] as usize],
                ]
            })
            .collect()
    }
}
