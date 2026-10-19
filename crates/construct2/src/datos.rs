// El proyecto de Construct 2 tal como lo exportó el original (data.js): tipos de objeto con sus
// animaciones, polígonos de colisión y comportamientos, y los layouts con sus capas e instancias.
// La estructura sigue al cargador del runtime (loadProject, Sprite.Type.onCreate, Layout, Layer).

use macroquad::math::{DVec2, dvec2};
use macroquad::prelude::{Color, Rect};
use serde_json::Value;

pub struct Cuadro {
    pub imagen: String,
    pub recorte: Rect,
    pub duracion: f64,
    /// Punto de origen (hotspot) en fracciones del tamaño.
    pub origen: DVec2,
    /// Polígono de colisión en fracciones del tamaño, relativo al origen. Vacío: el rectángulo.
    pub poligono: Vec<DVec2>,
    /// Puntos de imagen (image points), en fracciones del tamaño.
    pub puntos: Vec<DVec2>,
}

pub struct Animacion {
    pub nombre: String,
    pub velocidad: f64,
    pub bucle: bool,
    pub repeticiones: i32,
    pub repetir_desde: usize,
    pub ida_y_vuelta: bool,
    pub cuadros: Vec<Cuadro>,
}

/// Los comportamientos que el museo traduce. La exportación minificada no guarda de qué clase es
/// cada uno (solo su índice en la tabla de referencias del runtime), así que cada juego da la
/// correspondencia al leer el proyecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clase {
    Plataforma,
    Bala,
    Torreta,
    Movimiento,
    Envolver,
    Solido,
    Fisica,
    Desplazar,
}

/// Comportamiento de un tipo: su nombre en el proyecto y su clase (None si el port no la usa).
pub struct ComportamientoTipo {
    pub nombre: String,
    pub clase: Option<Clase>,
}

/// Tipo de objeto (t0, t1...: la exportación minificada perdió los nombres).
pub struct Tipo {
    pub nombre: String,
    /// Índice del plugin en la lista de plugins del proyecto.
    pub plugin: usize,
    /// La textura de los plugins que tienen una sola (Particles).
    pub textura: Option<String>,
    pub animaciones: Vec<Animacion>,
    pub comportamientos: Vec<ComportamientoTipo>,
}

impl Tipo {
    pub fn tiene(&self, clase: Clase) -> bool {
        self.comportamientos.iter().any(|c| c.clase == Some(clase))
    }
}

pub struct Instancia {
    pub tipo: usize,
    pub x: f64,
    pub y: f64,
    pub ancho: f64,
    pub alto: f64,
    pub angulo: f64,
    pub opacidad: f64,
    pub origen: DVec2,
    pub comportamientos: Vec<Value>,
    pub propiedades: Vec<Value>,
}

pub struct Capa {
    pub nombre: String,
    pub visible: bool,
    /// Parallax horizontal y vertical (1: se mueve con el scroll; 0: fija en pantalla).
    pub parallax: DVec2,
    pub fondo: Color,
    pub transparente: bool,
    pub instancias: Vec<Instancia>,
}

pub struct Layout {
    pub nombre: String,
    pub ancho: f64,
    pub alto: f64,
    /// Unbounded scrolling: el scroll no se limita a los bordes del layout.
    pub ilimitado: bool,
    pub capas: Vec<Capa>,
}

pub struct Proyecto {
    pub ancho: f64,
    pub alto: f64,
    pub primer_layout: String,
    pub tipos: Vec<Tipo>,
    pub layouts: Vec<Layout>,
}

fn real(v: &Value) -> f64 {
    v.as_f64().unwrap_or_default()
}

fn texto(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_owned()
}

impl Proyecto {
    /// `clases`: índice del constructor del comportamiento en la tabla de referencias del runtime
    /// (el segundo campo de cada comportamiento en data.js) y su clase.
    pub fn leer(data_js: &str, clases: &[(u64, Clase)]) -> Self {
        let json: Value =
            serde_json::from_str(data_js.trim_start_matches('\u{feff}')).expect("data.js");
        let p = &json["project"];
        let tipos = p[3]
            .as_array()
            .expect("tipos")
            .iter()
            .map(|t| Tipo {
                nombre: texto(&t[0]),
                plugin: t[1].as_u64().unwrap_or_default() as usize,
                textura: t[6].get(0).and_then(Value::as_str).map(str::to_owned),
                animaciones: t[7]
                    .as_array()
                    .map(|anims| anims.iter().map(leer_animacion).collect())
                    .unwrap_or_default(),
                comportamientos: t[8]
                    .as_array()
                    .map(|cs| {
                        cs.iter()
                            .map(|c| ComportamientoTipo {
                                nombre: texto(&c[0]),
                                clase: clases
                                    .iter()
                                    .find(|(i, _)| c[1].as_u64() == Some(*i))
                                    .map(|(_, clase)| *clase),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect();
        let layouts = p[5]
            .as_array()
            .expect("layouts")
            .iter()
            .map(|l| Layout {
                nombre: texto(&l[0]),
                ancho: real(&l[1]),
                alto: real(&l[2]),
                ilimitado: l[3].as_bool().unwrap_or(false),
                capas: l[6]
                    .as_array()
                    .expect("capas")
                    .iter()
                    .map(|c| {
                        let fondo = &c[4];
                        Capa {
                            nombre: texto(&c[0]),
                            visible: c[3].as_bool().unwrap_or(true),
                            parallax: dvec2(real(&c[6]), real(&c[7])),
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
            primer_layout: texto(&p[1]),
            tipos,
            layouts,
        }
    }

    /// La primera instancia de un tipo en el proyecto (en el orden de los layouts, sus capas y
    /// sus instancias): CreateObject la usa como modelo.
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
        nombre: texto(&a[0]),
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
                imagen: texto(&c[0]),
                recorte: Rect::new(
                    real(&c[2]) as f32,
                    real(&c[3]) as f32,
                    real(&c[4]) as f32,
                    real(&c[5]) as f32,
                ),
                duracion: real(&c[6]),
                origen: dvec2(real(&c[7]), real(&c[8])),
                poligono: c[10]
                    .as_array()
                    .map(|p| {
                        p.chunks(2)
                            .map(|xy| dvec2(real(&xy[0]), real(&xy[1])))
                            .collect()
                    })
                    .unwrap_or_default(),
                puntos: c[9]
                    .as_array()
                    .map(|ps| ps.iter().map(|p| dvec2(real(&p[1]), real(&p[2]))).collect())
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
        angulo: real(&w[6]),
        opacidad: real(&w[7]),
        origen: dvec2(real(&w[8]), real(&w[9])),
        comportamientos: i[4].as_array().cloned().unwrap_or_default(),
        propiedades: i
            .get(5)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    }
}
