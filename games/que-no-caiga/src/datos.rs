// El proyecto de Construct 2 tal como lo exportó el original (data.js): tipos de objeto con sus
// animaciones y polígonos de colisión, y los layouts con sus capas e instancias. La estructura
// sigue al cargador del runtime (loadProject, Sprite.Type.onCreate, Layout, Layer).

use macroquad::prelude::*;
use serde_json::Value;

pub struct Cuadro {
    pub imagen: String,
    pub recorte: Rect,
    pub duracion: f32,
    /// Punto de origen (hotspot) en fracciones del tamaño.
    pub origen: Vec2,
    /// Polígono de colisión en fracciones del tamaño, relativo al origen. Vacío: el rectángulo.
    pub poligono: Vec<Vec2>,
}

pub struct Animacion {
    pub nombre: String,
    pub velocidad: f32,
    pub bucle: bool,
    pub repeticiones: i32,
    pub repetir_desde: usize,
    pub ida_y_vuelta: bool,
    pub cuadros: Vec<Cuadro>,
}

/// Tipo de objeto (t0, t1...: la exportación minificada perdió los nombres).
pub struct Tipo {
    pub animaciones: Vec<Animacion>,
}

pub struct Instancia {
    pub tipo: usize,
    pub x: f32,
    pub y: f32,
    pub ancho: f32,
    pub alto: f32,
    pub opacidad: f32,
    pub origen: Vec2,
    pub comportamientos: Vec<Value>,
    pub propiedades: Vec<Value>,
}

pub struct Capa {
    pub visible: bool,
    pub fondo: Color,
    pub transparente: bool,
    pub instancias: Vec<Instancia>,
}

pub struct Layout {
    pub nombre: String,
    pub capas: Vec<Capa>,
}

pub struct Proyecto {
    pub ancho: f32,
    pub alto: f32,
    pub primer_layout: String,
    pub tipos: Vec<Tipo>,
    pub layouts: Vec<Layout>,
}

fn real(v: &Value) -> f32 {
    v.as_f64().unwrap_or_default() as f32
}

impl Proyecto {
    pub fn leer(data_js: &str) -> Self {
        let json: Value =
            serde_json::from_str(data_js.trim_start_matches('\u{feff}')).expect("data.js");
        let p = &json["project"];
        let tipos = p[3]
            .as_array()
            .expect("tipos")
            .iter()
            .map(|t| Tipo {
                animaciones: t[7]
                    .as_array()
                    .map(|anims| anims.iter().map(leer_animacion).collect())
                    .unwrap_or_default(),
            })
            .collect();
        let layouts = p[5]
            .as_array()
            .expect("layouts")
            .iter()
            .map(|l| Layout {
                nombre: l[0].as_str().unwrap_or_default().to_owned(),
                capas: l[6]
                    .as_array()
                    .expect("capas")
                    .iter()
                    .map(|c| {
                        let fondo = &c[4];
                        Capa {
                            visible: c[3].as_bool().unwrap_or(true),
                            fondo: Color::from_rgba(
                                fondo[0].as_u64().unwrap_or(255) as u8,
                                fondo[1].as_u64().unwrap_or(255) as u8,
                                fondo[2].as_u64().unwrap_or(255) as u8,
                                255,
                            ),
                            transparente: c[5].as_bool().unwrap_or(true),
                            instancias: c[14]
                                .as_array()
                                .expect("instancias")
                                .iter()
                                .map(leer_instancia)
                                .collect(),
                        }
                    })
                    .collect(),
            })
            .collect();
        Self {
            ancho: real(&p[10]),
            alto: real(&p[11]),
            primer_layout: p[1].as_str().unwrap_or_default().to_owned(),
            tipos,
            layouts,
        }
    }

    /// La primera instancia de un tipo en el proyecto: CreateObject la usa como modelo.
    pub fn instancia_por_defecto(&self, tipo: usize) -> &Instancia {
        self.layouts
            .iter()
            .flat_map(|l| &l.capas)
            .flat_map(|c| &c.instancias)
            .find(|i| i.tipo == tipo)
            .expect("instancia por defecto")
    }
}

/// Sprite.Type.onCreate: [nombre, velocidad, bucle, repeticiones, repetir desde, ida y vuelta,
/// sid, cuadros]; cada cuadro es [archivo, bytes, x, y, ancho, alto, duración, origen x, origen
/// y, puntos, polígono, formato].
fn leer_animacion(a: &Value) -> Animacion {
    Animacion {
        nombre: a[0].as_str().unwrap_or_default().to_owned(),
        velocidad: real(&a[1]),
        bucle: a[2].as_bool().unwrap_or(false),
        repeticiones: a[3].as_i64().unwrap_or(1) as i32,
        repetir_desde: a[4].as_u64().unwrap_or(0) as usize,
        ida_y_vuelta: a[5].as_bool().unwrap_or(false),
        cuadros: a[7]
            .as_array()
            .expect("cuadros")
            .iter()
            .map(|c| Cuadro {
                imagen: c[0].as_str().unwrap_or_default().to_owned(),
                recorte: Rect::new(real(&c[2]), real(&c[3]), real(&c[4]), real(&c[5])),
                duracion: real(&c[6]),
                origen: vec2(real(&c[7]), real(&c[8])),
                poligono: c[10]
                    .as_array()
                    .map(|p| {
                        p.chunks(2)
                            .map(|xy| vec2(real(&xy[0]), real(&xy[1])))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect(),
    }
}

/// Instancia de layout: [[x, y, z, ancho, alto, profundidad, ángulo, opacidad, origen x, origen y,
/// mezcla, ...], tipo, uid, variables, comportamientos, propiedades].
fn leer_instancia(i: &Value) -> Instancia {
    let w = &i[0];
    Instancia {
        tipo: i[1].as_u64().expect("tipo") as usize,
        x: real(&w[0]),
        y: real(&w[1]),
        ancho: real(&w[3]),
        alto: real(&w[4]),
        opacidad: real(&w[7]),
        origen: vec2(real(&w[8]), real(&w[9])),
        comportamientos: i[4].as_array().cloned().unwrap_or_default(),
        propiedades: i
            .get(5)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    }
}
