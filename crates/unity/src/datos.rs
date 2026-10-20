// El proyecto extraído del original (escena.json): ajustes, recursos, la escena y los prefabs. Lo
// escribe la herramienta de extracción de cada juego con UnityPy (por ejemplo
// games/super-happy-jump/tools/extraer.py); los nombres de los campos de Unity se traducen allí.

use std::collections::HashMap;

use serde::Deserialize;
pub use serde_json::Value;

#[derive(Deserialize)]
pub struct Proyecto {
    pub version: String,
    pub producto: String,
    pub pantalla_web: [u32; 2],
    pub tiempo: AjustesTiempo,
    pub fisica2d: AjustesFisica2d,
    /// Sorting layers en su orden de dibujo.
    pub capas_orden: Vec<String>,
    pub texturas: HashMap<String, [u32; 2]>,
    pub sprites: HashMap<String, DatosSprite>,
    pub clips: HashMap<String, Clip>,
    pub controladores: HashMap<String, Controlador>,
    pub materiales: HashMap<String, DatosMaterial>,
    pub materiales_fisicos: HashMap<String, MaterialFisico>,
    pub estilos: HashMap<String, Estilo>,
    pub fuentes: HashMap<String, DatosFuente>,
    pub escena: Vec<DatosObjeto>,
    pub prefabs: HashMap<String, Vec<DatosObjeto>>,
}

impl Proyecto {
    pub fn leer(json: &str) -> Self {
        serde_json::from_str(json).expect("escena.json")
    }
}

#[derive(Deserialize, Clone, Copy)]
pub struct AjustesTiempo {
    pub paso_fijo: f32,
    pub paso_maximo: f32,
    pub escala: f32,
}

#[derive(Deserialize, Clone)]
pub struct AjustesFisica2d {
    pub gravedad: [f32; 2],
    pub iteraciones_velocidad: i32,
    pub iteraciones_posicion: i32,
    pub rayos_tocan_triggers: bool,
    /// m_LayerCollisionMatrix: un bit por capa con la que choca cada capa.
    pub matriz: Vec<u32>,
}

#[derive(Deserialize, Clone)]
pub struct DatosSprite {
    pub textura: String,
    /// Rectángulo en píxeles de la textura, con el origen arriba a la izquierda.
    pub rect: [f32; 4],
    /// Pivote en fracciones del rectángulo, desde abajo a la izquierda.
    pub pivote: [f32; 2],
    /// Tamaño en unidades del mundo.
    pub tamano: [f32; 2],
    pub ppu: f32,
}

#[derive(Deserialize, Clone)]
pub struct Clip {
    pub fps: f32,
    /// m_StopTime: la duración del clip.
    pub fin: f32,
    pub bucle: bool,
    /// Curva de sprite: (tiempo, sprite) con interpolación por escalón.
    pub claves: Vec<(f32, String)>,
}

#[derive(Deserialize, Clone)]
pub struct Controlador {
    pub parametros: Vec<Parametro>,
    pub estados: Vec<Estado>,
    pub defecto: usize,
    pub desde_cualquiera: Vec<Transicion>,
}

#[derive(Deserialize, Clone)]
pub struct Parametro {
    pub nombre: String,
    pub tipo: String,
    pub defecto: Value,
}

#[derive(Deserialize, Clone)]
pub struct Estado {
    pub nombre: String,
    pub clip: String,
    pub velocidad: f32,
    pub bucle: bool,
    pub transiciones: Vec<Transicion>,
}

#[derive(Deserialize, Clone)]
pub struct Transicion {
    pub destino: usize,
    /// En tiempo normalizado del estado de origen (Unity 4 no tiene duraciones fijas).
    pub duracion: f32,
    pub desfase: f32,
    pub atomica: bool,
    pub a_si_misma: bool,
    pub condiciones: Vec<Condicion>,
}

#[derive(Deserialize, Clone)]
pub struct Condicion {
    /// AnimatorConditionMode: 1 If, 2 IfNot, 3 Greater, 4 Less, 6 Equals, 7 NotEqual.
    pub modo: u32,
    pub parametro: Option<String>,
    pub umbral: f32,
}

#[derive(Deserialize, Clone)]
pub struct DatosMaterial {
    pub shader: Option<String>,
    pub textura: Option<String>,
    pub tinte: Option<[f32; 4]>,
    pub color: Option<[f32; 4]>,
}

/// PhysicsMaterial2D.
#[derive(Deserialize, Clone, Copy)]
pub struct MaterialFisico {
    pub friccion: f32,
    pub rebote: f32,
}

#[derive(Deserialize, Clone)]
pub struct EstadoEstilo {
    pub fondo: Option<String>,
    pub color: [f32; 4],
}

#[derive(Deserialize, Clone)]
pub struct Estilo {
    pub normal: EstadoEstilo,
    pub hover: EstadoEstilo,
    pub active: EstadoEstilo,
    /// Izquierda, derecha, arriba, abajo.
    pub borde: [f32; 4],
    pub relleno: [f32; 4],
    pub fuente: Option<String>,
    pub tamano: u32,
    /// FontStyle: 0 normal, 1 negrita, 2 cursiva, 3 negrita y cursiva.
    pub estilo: u32,
    /// TextAnchor: 0 UpperLeft ... 4 MiddleCenter ... 8 LowerRight.
    pub alineacion: u32,
    pub ajuste: bool,
    /// TextClipping: 0 Overflow, 1 Clip.
    pub recorte: u32,
    pub desplazamiento: [f32; 2],
}

#[derive(Deserialize, Clone)]
pub struct DatosFuente {
    pub tamano: f32,
    pub ascenso: f32,
    pub interlineado: f32,
    pub nombres: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct DatosObjeto {
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
    pub componentes: Vec<Componente>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "tipo")]
pub enum Componente {
    SpriteRenderer {
        activo: bool,
        sprite: Option<String>,
        color: [f32; 4],
        capa_orden: String,
        orden: i32,
    },
    BoxCollider2D {
        activo: bool,
        trigger: bool,
        centro: [f32; 2],
        material: Option<String>,
        tamano: [f32; 2],
    },
    CircleCollider2D {
        activo: bool,
        trigger: bool,
        centro: [f32; 2],
        material: Option<String>,
        radio: f32,
    },
    Rigidbody2D {
        masa: f32,
        arrastre: f32,
        arrastre_angular: f32,
        escala_gravedad: f32,
        angulo_fijo: bool,
        cinematico: bool,
        sueno: u32,
    },
    Animator {
        activo: bool,
        controlador: String,
    },
    AudioSource {
        activo: bool,
        clip: Option<String>,
        al_despertar: bool,
        volumen: f32,
        bucle: bool,
        mudo: bool,
    },
    MonoBehaviour {
        activo: bool,
        script: String,
        campos: Value,
    },
    Camera {
        activo: bool,
        fondo: [f32; 4],
        ortografica: bool,
        tamano_ortografico: f32,
        campo_vision: f32,
        cerca: f32,
        lejos: f32,
    },
    ParticleSystem(Box<DatosParticulas>),
    ParticleSystemRenderer {
        activo: bool,
        material: Option<String>,
        /// ParticleSystemRenderMode: 0 Billboard, 1 Stretch, 2 HorizontalBillboard,
        /// 3 VerticalBillboard, 4 Mesh.
        modo: u32,
        escala_largo: f32,
        escala_velocidad: f32,
        tamano_maximo: f32,
        capa_orden: String,
        orden: i32,
    },
    /// MeshRenderer, MeshFilter, MeshCollider, GUILayer, FlareLayer, AudioListener...: no se
    /// traducen.
    #[serde(other)]
    Otro,
}

#[derive(Deserialize, Clone)]
pub struct DatosParticulas {
    pub duracion: f32,
    pub retraso: f32,
    pub velocidad_simulacion: f32,
    pub bucle: bool,
    pub precalentar: bool,
    pub al_despertar: bool,
    /// moveWithTransform: las partículas viven en el espacio local del emisor.
    pub espacio_local: bool,
    pub inicial: ModuloInicial,
    pub forma: Option<ModuloForma>,
    pub emision: Option<ModuloEmision>,
    pub tamano_en_vida: Option<MinMaxCurva>,
    pub color_en_vida: Option<MinMaxGradiente>,
    pub rotacion_en_vida: Option<MinMaxCurva>,
    pub hoja: Option<ModuloHoja>,
    pub velocidad_en_vida: Option<ModuloVelocidad>,
    #[serde(default)]
    pub sin_traducir: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct ModuloInicial {
    pub vida: MinMaxCurva,
    pub rapidez: MinMaxCurva,
    pub color: MinMaxGradiente,
    pub tamano: MinMaxCurva,
    pub rotacion: MinMaxCurva,
    pub gravedad: f32,
    pub heredar_velocidad: f32,
    pub maximo: usize,
}

#[derive(Deserialize, Clone)]
pub struct ModuloForma {
    /// ParticleSystemShapeType de Unity 4: 0 Sphere, 1 SphereShell, 2 Hemisphere,
    /// 3 HemisphereShell, 4 Cone, 5 Box...
    pub tipo: u32,
    pub radio: f32,
    pub angulo: f32,
    pub largo: f32,
    pub caja: [f32; 3],
    pub direccion_aleatoria: bool,
}

#[derive(Deserialize, Clone)]
pub struct ModuloEmision {
    pub tipo: u32,
    pub ritmo: MinMaxCurva,
    /// Ráfagas: (tiempo, cantidad).
    pub rafagas: Vec<(f32, u32)>,
}

#[derive(Deserialize, Clone)]
pub struct ModuloHoja {
    pub cuadro: MinMaxCurva,
    pub x: u32,
    pub y: u32,
    pub tipo: u32,
    pub fila: u32,
    pub ciclos: f32,
    pub fila_aleatoria: bool,
}

#[derive(Deserialize, Clone)]
pub struct ModuloVelocidad {
    pub x: MinMaxCurva,
    pub y: MinMaxCurva,
    pub z: MinMaxCurva,
    pub mundo: bool,
}

/// MinMaxCurve: `modo` 0 constante, 1 curva, 2 al azar entre dos curvas, 3 al azar entre dos
/// constantes (los primeros valores de `min` y `max`). Todo se multiplica por `escalar`.
#[derive(Deserialize, Clone)]
pub struct MinMaxCurva {
    pub modo: u32,
    pub escalar: f32,
    /// Claves (tiempo, valor, tangente de entrada, tangente de salida).
    pub max: Vec<[f32; 4]>,
    pub min: Vec<[f32; 4]>,
}

/// MinMaxGradient: `modo` 0 color, 1 gradiente, 2 al azar entre dos colores, 3 al azar entre dos
/// gradientes.
#[derive(Deserialize, Clone)]
pub struct MinMaxGradiente {
    pub modo: u32,
    pub max_color: [f32; 4],
    pub min_color: [f32; 4],
    pub max: Gradiente,
    pub min: Gradiente,
}

#[derive(Deserialize, Clone)]
pub struct Gradiente {
    /// (tiempo, r, g, b).
    pub colores: Vec<[f32; 4]>,
    /// (tiempo, a).
    pub alfas: Vec<[f32; 2]>,
}
