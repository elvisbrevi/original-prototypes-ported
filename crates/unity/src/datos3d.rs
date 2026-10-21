// El proyecto de un juego 3D de Unity 3.x extraído con UnityPy (escena.json): ajustes, escenas,
// materiales con su shader, mallas, la piel de la GUI, fuentes de mapa de bits, el destello de las
// luces y el terreno. Lo escribe la herramienta de extracción del juego (por ejemplo
// games/go-ball/tools/extraer.py); los nombres de los campos de Unity se traducen allí.

use std::collections::HashMap;

use serde::Deserialize;
pub use serde_json::Value;

use crate::datos::AjustesTiempo;

#[derive(Deserialize)]
pub struct Proyecto3d {
    pub version: String,
    pub producto: String,
    /// La resolución por defecto del build de escritorio.
    pub pantalla: [u32; 2],
    pub tiempo: AjustesTiempo,
    pub fisica: AjustesFisica,
    pub calidad: Calidad,
    pub ejes: Vec<Eje>,
    /// Los nombres de las escenas en el orden de BuildSettings.
    pub niveles: Vec<String>,
    pub escenas: Vec<DatosEscena>,
    pub materiales: HashMap<String, DatosMaterial>,
    pub mallas: HashMap<String, InfoMalla>,
    pub piel: Piel,
    pub fuentes: HashMap<String, FuenteMapa>,
    pub destello: Destello,
    pub terreno: DatosTerreno,
    pub audio: HashMap<String, DatosClip>,
    pub texturas: HashMap<String, DatosTextura>,
}

impl Proyecto3d {
    pub fn leer(json: &str) -> Self {
        serde_json::from_str(json).expect("escena.json")
    }

    /// El índice de la escena con ese nombre (Application.LoadLevel(string)).
    pub fn nivel(&self, nombre: &str) -> Option<usize> {
        self.niveles.iter().position(|n| n == nombre)
    }
}

/// PhysicsManager.
#[derive(Deserialize, Clone)]
pub struct AjustesFisica {
    pub gravedad: [f32; 3],
    pub umbral_rebote: f32,
    pub velocidad_dormir: f32,
    pub velocidad_angular_dormir: f32,
    pub velocidad_angular_maxima: f32,
    /// minPenetrationForPenalty: el «skin width» de PhysX.
    pub penetracion_minima: f32,
    pub iteraciones: u32,
    pub rayos_tocan_triggers: bool,
}

/// El nivel de QualitySettings con que arranca el build.
#[derive(Deserialize, Clone)]
pub struct Calidad {
    pub nivel: String,
    pub luces_pixel: u32,
    pub antialiasing: u32,
    pub vegetacion_suave: bool,
    pub particulas_suaves: bool,
}

/// Una entrada del InputManager.
#[derive(Deserialize, Clone)]
pub struct Eje {
    pub nombre: String,
    pub negativo: String,
    pub positivo: String,
    pub negativo_alt: String,
    pub positivo_alt: String,
    pub gravedad: f32,
    pub zona_muerta: f32,
    pub sensibilidad: f32,
    pub salto: bool,
    pub invertir: bool,
    /// 0: teclas o botones del ratón; 1: movimiento del ratón; 2: eje de mando.
    pub tipo: u32,
}

#[derive(Deserialize)]
pub struct DatosEscena {
    pub nombre: String,
    pub ajustes: AjustesEscena,
    pub objetos: Vec<DatosObjeto3d>,
}

/// RenderSettings.
#[derive(Deserialize, Clone)]
pub struct AjustesEscena {
    pub ambiente: [f32; 4],
    pub niebla: bool,
    /// El material del skybox (si la cámara borra con él).
    pub cielo: Option<String>,
    pub fuerza_destello: f32,
}

#[derive(Deserialize, Clone)]
pub struct DatosMaterial {
    pub nombre: String,
    /// El nombre con que se declara el shader (Shader "Diffuse" {...}).
    pub shader: Option<String>,
    pub texturas: HashMap<String, PropiedadTextura>,
    pub flotantes: HashMap<String, f32>,
    pub colores: HashMap<String, [f32; 4]>,
}

impl DatosMaterial {
    pub fn color(&self, nombre: &str, defecto: [f32; 4]) -> [f32; 4] {
        self.colores.get(nombre).copied().unwrap_or(defecto)
    }

    pub fn flotante(&self, nombre: &str, defecto: f32) -> f32 {
        self.flotantes.get(nombre).copied().unwrap_or(defecto)
    }

    pub fn textura(&self, nombre: &str) -> Option<&PropiedadTextura> {
        self.texturas.get(nombre).filter(|t| t.textura.is_some())
    }
}

#[derive(Deserialize, Clone)]
pub struct PropiedadTextura {
    pub textura: Option<String>,
    pub escala: [f32; 2],
    pub desplazamiento: [f32; 2],
}

#[derive(Deserialize, Clone)]
pub struct InfoMalla {
    pub vertices: usize,
    pub submallas: Vec<usize>,
    pub colision: usize,
    pub centro: [f32; 3],
    pub extension: [f32; 3],
}

#[derive(Deserialize, Clone)]
pub struct DatosTextura {
    pub tamano: [u32; 2],
    pub mipmaps: bool,
    /// FilterMode: 0 Point, 1 Bilinear, 2 Trilinear.
    pub filtro: u32,
    pub repetir: bool,
    #[serde(default)]
    pub cubemap: bool,
}

#[derive(Deserialize, Clone)]
pub struct DatosClip {
    pub archivo: String,
    #[serde(rename = "3d")]
    pub tres_d: bool,
    /// En segundos (para AudioSource.isPlaying).
    pub duracion: f32,
}

/// La GUISkin del juego.
#[derive(Deserialize, Clone)]
pub struct Piel {
    pub fuente: Option<String>,
    pub estilos: HashMap<String, EstiloPiel>,
}

#[derive(Deserialize, Clone)]
pub struct EstadoPiel {
    pub fondo: Option<String>,
    pub color: [f32; 4],
}

/// Un GUIStyle de Unity 3.x.
#[derive(Deserialize, Clone)]
pub struct EstiloPiel {
    pub nombre: String,
    pub normal: EstadoPiel,
    pub hover: EstadoPiel,
    pub active: EstadoPiel,
    pub focused: EstadoPiel,
    pub on_normal: EstadoPiel,
    pub on_hover: EstadoPiel,
    pub on_active: EstadoPiel,
    pub on_focused: EstadoPiel,
    /// Izquierda, derecha, arriba, abajo.
    pub borde: [i32; 4],
    pub margen: [i32; 4],
    pub relleno: [i32; 4],
    pub desborde: [i32; 4],
    pub fuente: Option<String>,
    pub posicion_imagen: u32,
    /// TextAnchor: 0 UpperLeft ... 8 LowerRight.
    pub alineacion: u32,
    pub ajuste: bool,
    pub recorte: u32,
    pub desplazamiento: [f32; 2],
    pub ancho_fijo: f32,
    pub alto_fijo: f32,
    pub tamano: u32,
    pub estilo: u32,
}

/// Una fuente de mapa de bits de Unity 3.x: la textura con los caracteres y, por carácter, su
/// rectángulo en la textura (uv, con el origen abajo), su rectángulo de dibujo respecto al punto de
/// la línea (vert, en píxeles, con y hacia arriba) y su avance.
#[derive(Deserialize, Clone)]
pub struct FuenteMapa {
    pub textura: Option<String>,
    pub interlineado: f32,
    pub kerning: f32,
    pub caracteres: HashMap<String, Caracter>,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Caracter {
    pub uv: [f32; 4],
    pub vert: [f32; 4],
    pub ancho: f32,
}

/// Un LensFlare (Flare): elementos sobre la línea que va de la luz al centro de la pantalla.
#[derive(Deserialize, Clone)]
pub struct Destello {
    pub nombre: String,
    pub textura: Option<String>,
    /// FlareLayout: 0 «1 Large 4 Small», 1 «1 Large 2 Medium 8 Small», 2 «1 Texture», 3 «2x2 Grid»,
    /// 4 «3x3 Grid», 5 «4x4 Grid».
    pub disposicion: u32,
    pub elementos: Vec<ElementoDestello>,
}

#[derive(Deserialize, Clone)]
pub struct ElementoDestello {
    pub imagen: u32,
    pub posicion: f32,
    pub tamano: f32,
    pub color: [f32; 4],
    pub color_luz: bool,
    pub girar: bool,
    pub zoom: bool,
    pub fundido: bool,
}

#[derive(Deserialize, Clone)]
pub struct DatosTerreno {
    pub nombre: String,
    /// Muestras del mapa de alturas (ancho en x, alto en z).
    pub alturas: [u32; 2],
    /// Tamaño de una muestra en x, la altura máxima y el tamaño en z.
    pub escala: [f32; 3],
    pub capas: Vec<CapaTerreno>,
    pub mezcla: Vec<Option<String>>,
    pub resolucion_mezcla: u32,
    pub detalles: Detalles,
}

#[derive(Deserialize, Clone)]
pub struct CapaTerreno {
    pub textura: Option<String>,
    pub tamano: [f32; 2],
    pub desplazamiento: [f32; 2],
}

#[derive(Deserialize, Clone)]
pub struct Detalles {
    /// Parches por lado.
    pub parches: u32,
    /// Muestras por lado de cada parche.
    pub muestras: u32,
    pub prototipos: Vec<PrototipoDetalle>,
    pub tinte: [f32; 4],
    pub fuerza: f32,
    pub cantidad: f32,
    pub rapidez: f32,
}

#[derive(Deserialize, Clone)]
pub struct PrototipoDetalle {
    pub textura: Option<String>,
    pub ancho: [f32; 2],
    pub alto: [f32; 2],
    pub ruido: f32,
    pub sano: [f32; 4],
    pub seco: [f32; 4],
    /// DetailRenderMode: 0 GrassBillboard, 1 VertexLit, 2 Grass.
    pub modo: u32,
}

/// Un parche del terreno con detalles: por cada capa, la cantidad de objetos de cada muestra.
#[derive(Deserialize, Clone)]
pub struct ParcheDetalle {
    pub x: u32,
    pub z: u32,
    pub capas: Vec<u32>,
    pub cantidades: Vec<u8>,
}

#[derive(Deserialize, Clone)]
pub struct DatosObjeto3d {
    pub id: i64,
    pub nombre: String,
    pub activo: bool,
    pub capa: u32,
    pub etiqueta: String,
    pub posicion: [f32; 3],
    pub rotacion: [f32; 4],
    pub escala: [f32; 3],
    pub padre: Option<i64>,
    pub hijos: Vec<i64>,
    pub componentes: Vec<Componente3d>,
}

/// Una referencia de un campo de script: un objeto de la escena (o uno de sus componentes) o un
/// recurso por su nombre.
pub fn referencia_objeto(v: &Value) -> Option<i64> {
    v.get("objeto").and_then(Value::as_i64)
}

#[derive(Deserialize, Clone)]
#[serde(tag = "tipo")]
pub enum Componente3d {
    MeshFilter {
        malla: Option<String>,
    },
    MeshRenderer {
        activo: bool,
        materiales: Vec<Option<String>>,
    },
    MeshCollider {
        activo: bool,
        trigger: bool,
        malla: Option<String>,
        convexo: bool,
    },
    BoxCollider {
        activo: bool,
        trigger: bool,
        centro: [f32; 3],
        tamano: [f32; 3],
    },
    SphereCollider {
        activo: bool,
        trigger: bool,
        centro: [f32; 3],
        radio: f32,
    },
    TerrainCollider {
        activo: bool,
    },
    Rigidbody {
        masa: f32,
        arrastre: f32,
        arrastre_angular: f32,
        gravedad: bool,
        cinematico: bool,
        /// RigidbodyConstraints: 2/4/8 congelan la posición en x/y/z, 16/32/64 la rotación.
        restricciones: u32,
    },
    Light {
        activo: bool,
        /// LightType: 0 Spot, 1 Directional, 2 Point.
        clase: u32,
        color: [f32; 4],
        intensidad: f32,
        rango: f32,
        destello: Option<String>,
    },
    Camera {
        activo: bool,
        /// CameraClearFlags: 1 Skybox, 2 SolidColor, 3 Depth, 4 Nothing.
        borrado: u32,
        fondo: [f32; 4],
        campo_vision: f32,
        cerca: f32,
        lejos: f32,
    },
    AudioSource {
        activo: bool,
        clip: Option<String>,
        al_despertar: bool,
        volumen: f32,
        bucle: bool,
        mudo: bool,
        distancia_minima: f32,
        distancia_maxima: f32,
        /// AudioRolloffMode: 0 Logarithmic, 1 Linear, 2 Custom.
        atenuacion: u32,
    },
    GUITexture {
        activo: bool,
        textura: Option<String>,
        color: [f32; 4],
        margen: [f32; 4],
    },
    EllipsoidParticleEmitter(Box<Emisor>),
    ParticleAnimator(Box<AnimadorParticulas>),
    ParticleRenderer {
        activo: bool,
        materiales: Vec<Option<String>>,
        /// ParticleRenderMode: 0 Billboard, 3 Stretch, ...
        estirar: u32,
        tamano_maximo: f32,
    },
    WorldParticleCollider {
        rebote: f32,
        perdida: f32,
        choca_con: u32,
        mensaje: bool,
        velocidad_minima: f32,
    },
    Animation {
        clips: Vec<String>,
    },
    MonoBehaviour {
        #[serde(default = "activo")]
        activo: bool,
        script: String,
        campos: Value,
    },
    /// GUILayer, FlareLayer, AudioListener: sin datos.
    #[serde(other)]
    Otro,
}

fn activo() -> bool {
    true
}

/// EllipsoidParticleEmitter.
#[derive(Deserialize, Clone)]
pub struct Emisor {
    pub activo: bool,
    pub emitir: bool,
    pub tamano: [f32; 2],
    pub energia: [f32; 2],
    pub emision: [f32; 2],
    pub velocidad_mundo: [f32; 3],
    pub velocidad_local: [f32; 3],
    pub velocidad_azar: [f32; 3],
    pub escala_velocidad_emisor: f32,
    pub velocidad_tangente: [f32; 3],
    pub velocidad_angular: f32,
    pub velocidad_angular_azar: f32,
    pub rotacion_azar: bool,
    pub mundo: bool,
    pub una_vez: bool,
    pub elipsoide: [f32; 3],
    pub rango_minimo: f32,
}

/// ParticleAnimator.
#[derive(Deserialize, Clone)]
pub struct AnimadorParticulas {
    pub animar_color: bool,
    pub colores: [[f32; 4]; 5],
    pub eje_mundo: [f32; 3],
    pub eje_local: [f32; 3],
    pub crecer: f32,
    pub fuerza_azar: [f32; 3],
    pub fuerza: [f32; 3],
    pub amortiguacion: f32,
    pub autodestruir: bool,
}
