// El dibujo 3D de Unity 3.x en el camino Forward, con miniquad directamente (búferes estáticos
// por malla, mipmaps, cubemaps): la cámara de perspectiva, el skybox, las colas de Unity (opacos
// 2000, alpha test 2450, las hojas de los árboles 2901 y los transparentes 3000, estos de atrás
// hacia adelante), los MeshRenderer con sus materiales (shaders3d), las partículas heredadas como
// billboards y los destellos de las luces. Unity ve el mundo con el eje z hacia adelante (mano
// izquierda); la vista lo invierte para usar las matrices de OpenGL, como el
// worldToCameraMatrix de Unity.

use std::collections::HashMap;

use macroquad::miniquad::{
    BlendFactor, BlendState, BlendValue, BufferId, BufferLayout, BufferSource, BufferType,
    BufferUsage, Bindings, Comparison, CullFace, Equation, FilterMode, FrontFaceOrder,
    MipmapFilterMode, PassAction, Pipeline, PipelineParams, RenderPass, RenderingBackend,
    ShaderMeta, ShaderSource, TextureAccess, TextureFormat, TextureId, TextureKind,
    TextureParams, TextureSource, TextureWrap, UniformBlockLayout, UniformDesc, UniformType,
    UniformsSource, VertexAttribute, VertexFormat,
};
use macroquad::prelude::{Color, Mat4, Quat, Vec2, Vec3, Vec4, vec2, vec3};

use crate::datos3d::{DatosMaterial, Proyecto3d};
use crate::escena3d::{Escena3d, Id};
use crate::malla::Malla;
use crate::shaders3d::*;
use crate::terreno::{Terreno, VerticeTerreno};

/// El orden de los vértices de las caras de adelante después de la vista y del giro vertical del
/// render target.
const FRENTE: FrontFaceOrder = FrontFaceOrder::CounterClockwise;

/// miniquad 0.4 apaga el depth test entero cuando un pipeline no escribe la profundidad (nunca
/// llama a glDepthMask). Los transparentes de Unity prueban la profundidad sin escribirla
/// (ZWrite Off): sus pipelines se crean con escritura y aquí se apaga la máscara mientras se
/// dibujan (en la web, glDepthMask lo agrega el index.html de crates/museo).
fn escribir_profundidad(si: bool) {
    use macroquad::miniquad::gl;
    unsafe { gl::glDepthMask(if si { gl::GL_TRUE } else { gl::GL_FALSE } as _) };
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertice {
    posicion: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
    tangente: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VerticeColor {
    pub posicion: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

struct MallaGpu {
    vertices: BufferId,
    submallas: Vec<(BufferId, i32)>,
    centro: Vec3,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tipo {
    /// Diffuse (y Bumped Specular, con brillo).
    Iluminado,
    Corteza,
    Hojas,
    /// Transparent/Cutout/Soft Edge Unlit.
    Recorte,
    /// My Shaders/AlphaSelfIllum.
    Nube,
    Diamante,
    Cielo,
    Particulas,
    /// Un shader que no se dibuja (la fuente de la GUI).
    Nada,
}

struct Material3d {
    tipo: Tipo,
    textura: TextureId,
    st: [f32; 4],
    color: [f32; 4],
    /// _SpecColor y el exponente (_Shininess * 128).
    especular: [f32; 4],
    /// _AO, _BaseLight, _Occlusion.
    oclusion: [f32; 4],
    corte: f32,
    /// _TintColor (partículas y skybox).
    tinte: [f32; 4],
    cubos: [TextureId; 2],
}

#[repr(C)]
struct UIluminado {
    mvp: [f32; 16],
    modelo: [f32; 16],
    normales: [f32; 16],
    st: [f32; 4],
    color: [f32; 4],
    ambiente: [f32; 4],
    luz_posicion: [f32; 4],
    luz_color: [f32; 4],
    especular: [f32; 4],
    camara: [f32; 4],
}

#[repr(C)]
struct UArbol {
    mvp: [f32; 16],
    modelo: [f32; 16],
    color: [f32; 4],
    ambiente: [f32; 4],
    luz_posicion: [f32; 4],
    luz_color: [f32; 4],
    oclusion: [f32; 4],
    hojas: [f32; 4],
    corte: [f32; 4],
}

#[repr(C)]
struct USimple {
    mvp: [f32; 16],
    st: [f32; 4],
    color: [f32; 4],
    recorte: [f32; 4],
}

#[repr(C)]
struct UDiamante {
    mvp: [f32; 16],
    modelo: [f32; 16],
    normales: [f32; 16],
    camara: [f32; 4],
    color: [f32; 4],
    pasada: [f32; 4],
}

#[repr(C)]
struct UColor {
    mvp: [f32; 16],
    tinte: [f32; 4],
}

#[repr(C)]
struct UTerreno {
    mvp: [f32; 16],
    repetir: [f32; 4],
    ambiente: [f32; 4],
    luz_posicion: [f32; 4],
    luz_color: [f32; 4],
}

#[repr(C)]
struct UPasto {
    mvp: [f32; 16],
    onda: [f32; 4],
    camara: [f32; 4],
    derecha: [f32; 4],
    arriba: [f32; 4],
    tinte: [f32; 4],
    ambiente: [f32; 4],
    luz_posicion: [f32; 4],
    luz_color: [f32; 4],
    corte: [f32; 4],
}

/// El terreno de la escena en la GPU: su malla, las capas y el pasto por prototipo.
struct TerrenoGpu {
    vertices: BufferId,
    indices: BufferId,
    n: i32,
    control: TextureId,
    capas: [TextureId; 2],
    repetir: [f32; 4],
    pasto: Vec<(BufferId, BufferId, i32, TextureId)>,
    /// _WavingTint, la fuerza y la cantidad de las ondas, su rapidez y la distancia de los
    /// detalles.
    tinte: [f32; 4],
    fuerza: f32,
    cantidad: f32,
    rapidez: f32,
    distancia: f32,
}

/// La cámara de perspectiva de Unity (Field of View vertical, en grados).
#[derive(Clone, Copy, Debug)]
pub struct Camara3d {
    pub posicion: Vec3,
    pub rotacion: Quat,
    pub campo_vision: f32,
    pub cerca: f32,
    pub lejos: f32,
    pub ancho: f32,
    pub alto: f32,
}

impl Camara3d {
    /// worldToCameraMatrix: la inversa de la pose con el eje z invertido.
    pub fn vista(&self) -> Mat4 {
        Mat4::from_scale(vec3(1.0, 1.0, -1.0))
            * Mat4::from_rotation_translation(self.rotacion, self.posicion).inverse()
    }

    pub fn proyeccion(&self) -> Mat4 {
        Mat4::perspective_rh_gl(
            self.campo_vision.to_radians(),
            self.ancho / self.alto,
            self.cerca,
            self.lejos,
        )
    }

    /// De mundo a la pantalla de un render target de macroquad (que guarda la imagen con la y
    /// invertida).
    fn matriz(&self) -> Mat4 {
        Mat4::from_scale(vec3(1.0, -1.0, 1.0)) * self.proyeccion() * self.vista()
    }

    /// Camera.WorldToScreenPoint: píxeles con el origen abajo a la izquierda y la distancia al
    /// plano de la cámara en z.
    pub fn a_pantalla(&self, p: Vec3) -> Vec3 {
        let v = self.vista().transform_point3(p);
        let c = self.proyeccion() * v.extend(1.0);
        let ndc = c.truncate() / c.w;
        vec3(
            (ndc.x + 1.0) / 2.0 * self.ancho,
            (ndc.y + 1.0) / 2.0 * self.alto,
            -v.z,
        )
    }

    /// Camera.ScreenToWorldPoint: un punto de la pantalla (origen abajo a la izquierda) a la
    /// distancia z del plano de la cámara.
    pub fn a_mundo(&self, p: Vec3) -> Vec3 {
        let t = (self.campo_vision.to_radians() / 2.0).tan();
        let x = (p.x / self.ancho * 2.0 - 1.0) * t * self.ancho / self.alto;
        let y = (p.y / self.alto * 2.0 - 1.0) * t;
        let local = vec3(x * p.z, y * p.z, p.z);
        self.posicion + self.rotacion * local
    }

    /// Camera.ScreenPointToRay: origen en el plano cercano y dirección.
    pub fn rayo(&self, p: Vec2) -> (Vec3, Vec3) {
        let cerca = self.a_mundo(p.extend(self.cerca));
        let lejos = self.a_mundo(p.extend(self.lejos));
        (cerca, (lejos - cerca).normalize())
    }
}

/// Una luz puntual para el dibujo.
#[derive(Clone, Copy)]
pub struct LuzPuntual {
    pub posicion: Vec3,
    /// color * intensidad.
    pub color: Vec3,
    pub rango: f32,
}

/// Un sistema de partículas para dibujar: material y (posición, tamaño, rotación en grados,
/// color) de cada partícula.
pub struct Billboards<'a> {
    pub material: &'a str,
    pub tamano_maximo: f32,
    pub particulas: Vec<(Vec3, f32, f32, Color)>,
}

/// Un cuadro en la pantalla (píxeles con el origen arriba a la izquierda) para los destellos.
pub struct CuadroPantalla {
    pub centro: Vec2,
    pub tamano: f32,
    pub angulo: f32,
    pub uv: [f32; 4],
    pub color: Color,
}

pub struct Render3d {
    pub ancho: f32,
    pub alto: f32,
    mallas: HashMap<String, MallaGpu>,
    texturas: HashMap<String, TextureId>,
    materiales: HashMap<String, Material3d>,
    blanca: TextureId,
    iluminado: Pipeline,
    corteza: Pipeline,
    hojas: Pipeline,
    recorte_opaco: Pipeline,
    recorte_suave: Pipeline,
    nube: Pipeline,
    diamante_atras: Pipeline,
    diamante_frente: Pipeline,
    cielo: Pipeline,
    particulas: Pipeline,
    aditivo: Pipeline,
    terreno_p: Pipeline,
    pasto_p: Pipeline,
    terreno: Option<TerrenoGpu>,
    /// Time.time, para las ondas del pasto.
    pub tiempo: f32,
    cubo: (BufferId, Vec<(BufferId, i32)>),
    dinamico: (BufferId, BufferId),
}

const MAX_CUADROS: usize = 4096;

fn uniformes(nombres: &[(&str, UniformType)]) -> UniformBlockLayout {
    UniformBlockLayout {
        uniforms: nombres
            .iter()
            .map(|(n, t)| UniformDesc::new(n, *t))
            .collect(),
    }
}

fn mezcla_alfa() -> Option<BlendState> {
    Some(BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    ))
}

fn mezcla_suma() -> Option<BlendState> {
    Some(BlendState::new(
        Equation::Add,
        BlendFactor::One,
        BlendFactor::One,
    ))
}

fn atributos_malla() -> Vec<VertexAttribute> {
    vec![
        VertexAttribute::new("posicion", VertexFormat::Float3),
        VertexAttribute::new("normal", VertexFormat::Float3),
        VertexAttribute::new("uv", VertexFormat::Float2),
        VertexAttribute::new("color", VertexFormat::Float4),
        VertexAttribute::new("tangente", VertexFormat::Float4),
    ]
}

fn atributos_color() -> Vec<VertexAttribute> {
    vec![
        VertexAttribute::new("posicion", VertexFormat::Float3),
        VertexAttribute::new("uv", VertexFormat::Float2),
        VertexAttribute::new("color", VertexFormat::Float4),
    ]
}

fn columnas(m: Mat4) -> [f32; 16] {
    m.to_cols_array()
}

fn v4(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

/// El shader de Unity de cada material, por su nombre.
fn tipo_de(shader: Option<&str>) -> Tipo {
    match shader.unwrap_or("") {
        "Diffuse" | "Bumped Specular" | "Specular" | "VertexLit" => Tipo::Iluminado,
        "Nature/Tree Soft Occlusion Bark" => Tipo::Corteza,
        "Nature/Tree Soft Occlusion Leaves" => Tipo::Hojas,
        "Transparent/Cutout/Soft Edge Unlit" => Tipo::Recorte,
        "My Shaders/AlphaSelfIllum" => Tipo::Nube,
        "FX/Diamond" => Tipo::Diamante,
        "RenderFX/Skybox" => Tipo::Cielo,
        s if s.starts_with("Particles/") => Tipo::Particulas,
        _ => Tipo::Nada,
    }
}

/// La cola de dibujo de Unity de cada tipo de shader.
fn cola(tipo: Tipo) -> i32 {
    match tipo {
        Tipo::Iluminado | Tipo::Corteza => 2000,
        Tipo::Recorte => 2450,
        Tipo::Hojas => 2901,
        _ => 3000,
    }
}

impl Render3d {
    /// Carga las mallas, las texturas (PNG, con mipmaps; los cubemaps como seis caras
    /// `<nombre>_+x.png`...) y los materiales del proyecto.
    pub fn nuevo(
        proyecto: &Proyecto3d,
        mallas: &HashMap<String, Malla>,
        archivos: &HashMap<String, &[u8]>,
        ancho: f32,
        alto: f32,
    ) -> Self {
        let ctx = unsafe { macroquad::window::get_internal_gl() }.quad_context;
        let blanca = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);
        let mut texturas = HashMap::new();
        for (nombre, datos) in &proyecto.texturas {
            let textura = if datos.cubemap {
                let caras: Vec<Vec<u8>> = ["+x", "-x", "+y", "-y", "+z", "-z"]
                    .iter()
                    .map(|c| {
                        let bytes = archivos[&format!("{nombre}_{c}")];
                        macroquad::texture::Image::from_file_with_format(bytes, None)
                            .expect("cara")
                            .bytes
                    })
                    .collect();
                let niveles: Vec<[&[u8]; 1]> = caras.iter().map(|c| [c.as_slice()]).collect();
                let refs: Vec<&[&[u8]]> = niveles.iter().map(|n| n.as_slice()).collect();
                ctx.new_texture(
                    TextureAccess::Static,
                    TextureSource::Array(&refs),
                    TextureParams {
                        kind: TextureKind::CubeMap,
                        format: TextureFormat::RGBA8,
                        wrap: TextureWrap::Clamp,
                        min_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        mipmap_filter: MipmapFilterMode::None,
                        width: datos.tamano[0],
                        height: datos.tamano[1],
                        allocate_mipmaps: false,
                        sample_count: 1,
                    },
                )
            } else {
                let Some(bytes) = archivos.get(nombre) else {
                    continue;
                };
                let imagen = macroquad::texture::Image::from_file_with_format(bytes, None)
                    .expect("textura");
                // Unity guarda las filas de abajo hacia arriba (la v = 0 de las UV es la fila de
                // abajo); el PNG va de arriba hacia abajo.
                let fila = imagen.width as usize * 4;
                let filas: Vec<u8> = imagen.bytes.chunks_exact(fila).rev().flatten().copied().collect();
                let t = ctx.new_texture(
                    TextureAccess::Static,
                    TextureSource::Bytes(&filas),
                    TextureParams {
                        kind: TextureKind::Texture2D,
                        format: TextureFormat::RGBA8,
                        wrap: if datos.repetir {
                            TextureWrap::Repeat
                        } else {
                            TextureWrap::Clamp
                        },
                        min_filter: if datos.filtro == 0 {
                            FilterMode::Nearest
                        } else {
                            FilterMode::Linear
                        },
                        mag_filter: if datos.filtro == 0 {
                            FilterMode::Nearest
                        } else {
                            FilterMode::Linear
                        },
                        mipmap_filter: if datos.mipmaps {
                            MipmapFilterMode::Linear
                        } else {
                            MipmapFilterMode::None
                        },
                        width: imagen.width as u32,
                        height: imagen.height as u32,
                        allocate_mipmaps: datos.mipmaps,
                        sample_count: 1,
                    },
                );
                if datos.mipmaps {
                    ctx.texture_generate_mipmaps(t);
                }
                t
            };
            texturas.insert(nombre.clone(), textura);
        }

        let mut gpu = HashMap::new();
        for (nombre, m) in mallas {
            let n = m.posiciones.len();
            let vertices: Vec<Vertice> = (0..n)
                .map(|i| {
                    let c = m.colores.get(i).copied().unwrap_or([255; 4]);
                    Vertice {
                        posicion: m.posiciones[i].to_array(),
                        normal: m.normales.get(i).copied().unwrap_or(Vec3::Y).to_array(),
                        uv: m.uv.get(i).copied().unwrap_or(Vec2::ZERO).to_array(),
                        color: c.map(|x| x as f32 / 255.0),
                        tangente: m.tangentes.get(i).copied().unwrap_or(Vec4::ZERO).to_array(),
                    }
                })
                .collect();
            let vb = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );
            let subs = m
                .submallas
                .iter()
                .map(|s| {
                    (
                        ctx.new_buffer(
                            BufferType::IndexBuffer,
                            BufferUsage::Immutable,
                            BufferSource::slice(s),
                        ),
                        s.len() as i32,
                    )
                })
                .collect();
            let info = &proyecto.mallas[nombre];
            gpu.insert(
                nombre.clone(),
                MallaGpu {
                    vertices: vb,
                    submallas: subs,
                    centro: Vec3::from(info.centro),
                },
            );
        }

        let materiales = proyecto
            .materiales
            .iter()
            .map(|(k, m)| (k.clone(), Self::material(m, &texturas, blanca)))
            .collect();

        let ilum = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_ILUMINADO,
                    fragment: &con_atenuacion(FS_ILUMINADO),
                },
                ShaderMeta {
                    images: vec!["textura".to_owned()],
                    uniforms: uniformes(&[
                        ("mvp", UniformType::Mat4),
                        ("modelo", UniformType::Mat4),
                        ("normales", UniformType::Mat4),
                        ("st", UniformType::Float4),
                        ("color", UniformType::Float4),
                        ("ambiente", UniformType::Float4),
                        ("luz_posicion", UniformType::Float4),
                        ("luz_color", UniformType::Float4),
                        ("especular", UniformType::Float4),
                        ("camara", UniformType::Float4),
                    ]),
                },
            )
            .expect("shader iluminado");
        let arbol = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_ARBOL,
                    fragment: FS_ARBOL,
                },
                ShaderMeta {
                    images: vec!["textura".to_owned()],
                    uniforms: uniformes(&[
                        ("mvp", UniformType::Mat4),
                        ("modelo", UniformType::Mat4),
                        ("color", UniformType::Float4),
                        ("ambiente", UniformType::Float4),
                        ("luz_posicion", UniformType::Float4),
                        ("luz_color", UniformType::Float4),
                        ("oclusion", UniformType::Float4),
                        ("hojas", UniformType::Float4),
                        ("corte", UniformType::Float4),
                    ]),
                },
            )
            .expect("shader árbol");
        let meta_simple = ShaderMeta {
            images: vec!["textura".to_owned()],
            uniforms: uniformes(&[
                ("mvp", UniformType::Mat4),
                ("st", UniformType::Float4),
                ("color", UniformType::Float4),
                ("recorte", UniformType::Float4),
            ]),
        };
        let simple = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_SIMPLE,
                    fragment: FS_SIMPLE,
                },
                meta_simple.clone(),
            )
            .expect("shader simple");
        let cielo = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_SIMPLE,
                    fragment: FS_CIELO,
                },
                meta_simple,
            )
            .expect("shader cielo");
        let diamante = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_DIAMANTE,
                    fragment: FS_DIAMANTE,
                },
                ShaderMeta {
                    images: vec!["refraccion".to_owned(), "reflexion".to_owned()],
                    uniforms: uniformes(&[
                        ("mvp", UniformType::Mat4),
                        ("modelo", UniformType::Mat4),
                        ("normales", UniformType::Mat4),
                        ("camara", UniformType::Float4),
                        ("color", UniformType::Float4),
                        ("pasada", UniformType::Float4),
                    ]),
                },
            )
            .expect("shader diamante");
        let color = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_COLOR,
                    fragment: FS_COLOR,
                },
                ShaderMeta {
                    images: vec!["textura".to_owned()],
                    uniforms: uniformes(&[
                        ("mvp", UniformType::Mat4),
                        ("tinte", UniformType::Float4),
                    ]),
                },
            )
            .expect("shader color");

        let terreno = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_TERRENO,
                    fragment: &con_atenuacion(FS_TERRENO),
                },
                ShaderMeta {
                    images: vec!["control".to_owned(), "capa0".to_owned(), "capa1".to_owned()],
                    uniforms: uniformes(&[
                        ("mvp", UniformType::Mat4),
                        ("repetir", UniformType::Float4),
                        ("ambiente", UniformType::Float4),
                        ("luz_posicion", UniformType::Float4),
                        ("luz_color", UniformType::Float4),
                    ]),
                },
            )
            .expect("shader terreno");
        let pasto = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VS_PASTO,
                    fragment: &con_atenuacion(FS_PASTO),
                },
                ShaderMeta {
                    images: vec!["textura".to_owned()],
                    uniforms: uniformes(&[
                        ("mvp", UniformType::Mat4),
                        ("onda", UniformType::Float4),
                        ("camara", UniformType::Float4),
                        ("derecha", UniformType::Float4),
                        ("arriba", UniformType::Float4),
                        ("tinte", UniformType::Float4),
                        ("ambiente", UniformType::Float4),
                        ("luz_posicion", UniformType::Float4),
                        ("luz_color", UniformType::Float4),
                        ("corte", UniformType::Float4),
                    ]),
                },
            )
            .expect("shader pasto");
        let base = PipelineParams {
            depth_test: Comparison::LessOrEqual,
            depth_write: true,
            front_face_order: FRENTE,
            cull_face: CullFace::Back,
            ..Default::default()
        };
        let malla = [BufferLayout::default()];
        let mut pipeline = |shader, attrs: Vec<VertexAttribute>, p: PipelineParams| {
            ctx.new_pipeline(&malla, &attrs, shader, p)
        };
        let iluminado = pipeline(ilum, atributos_malla(), base);
        let terreno_p = pipeline(terreno, atributos_malla(), base);
        let pasto_p = pipeline(
            pasto,
            atributos_malla(),
            PipelineParams {
                cull_face: CullFace::Nothing,
                ..base
            },
        );
        let corteza = pipeline(arbol, atributos_malla(), base);
        let hojas = pipeline(
            arbol,
            atributos_malla(),
            PipelineParams {
                cull_face: CullFace::Nothing,
                ..base
            },
        );
        let recorte_opaco = pipeline(
            simple,
            atributos_malla(),
            PipelineParams {
                cull_face: CullFace::Nothing,
                ..base
            },
        );
        // Los transparentes llevan depth_write en true para que miniquad deje el depth test, y
        // se dibujan con la máscara de profundidad apagada (escribir_profundidad).
        let recorte_suave = pipeline(
            simple,
            atributos_malla(),
            PipelineParams {
                cull_face: CullFace::Nothing,
                color_blend: mezcla_alfa(),
                ..base
            },
        );
        let nube = pipeline(
            simple,
            atributos_malla(),
            PipelineParams {
                color_blend: mezcla_alfa(),
                ..base
            },
        );
        let diamante_atras = pipeline(
            diamante,
            atributos_malla(),
            PipelineParams {
                cull_face: CullFace::Front,
                depth_write_offset: Some((-1.0, -1.0)),
                ..base
            },
        );
        let diamante_frente = pipeline(
            diamante,
            atributos_malla(),
            PipelineParams {
                color_blend: mezcla_suma(),
                ..base
            },
        );
        let cielo = pipeline(
            cielo,
            atributos_malla(),
            PipelineParams {
                depth_test: Comparison::Always,
                depth_write: false,
                cull_face: CullFace::Nothing,
                ..base
            },
        );
        let particulas = pipeline(
            color,
            atributos_color(),
            PipelineParams {
                cull_face: CullFace::Nothing,
                color_blend: mezcla_alfa(),
                ..base
            },
        );
        let aditivo = pipeline(
            color,
            atributos_color(),
            PipelineParams {
                depth_test: Comparison::Always,
                depth_write: false,
                cull_face: CullFace::Nothing,
                color_blend: mezcla_suma(),
                ..base
            },
        );

        // El cubo del skybox: seis caras vistas desde adentro (Front +z, Back -z, Left +x,
        // Right -x, Up +y, Down -y), cada una con su submalla para su textura.
        let caras: [(Vec3, Vec3, Vec3); 6] = [
            (Vec3::Z, Vec3::X, Vec3::Y),
            (-Vec3::Z, -Vec3::X, Vec3::Y),
            (Vec3::X, -Vec3::Z, Vec3::Y),
            (-Vec3::X, Vec3::Z, Vec3::Y),
            (Vec3::Y, Vec3::X, -Vec3::Z),
            (-Vec3::Y, Vec3::X, Vec3::Z),
        ];
        let mut vertices = Vec::new();
        let mut submallas = Vec::new();
        for (centro, derecha, arriba) in caras {
            let i = vertices.len() as u16;
            for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                let p = centro + derecha * (u * 2.0 - 1.0) + arriba * (v * 2.0 - 1.0);
                vertices.push(Vertice {
                    posicion: p.to_array(),
                    normal: [0.0; 3],
                    uv: [u, v],
                    color: [1.0; 4],
                    tangente: [0.0; 4],
                });
            }
            let indices = [i, i + 1, i + 2, i, i + 2, i + 3];
            submallas.push((
                ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&indices),
                ),
                6,
            ));
        }
        let cubo = (
            ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            ),
            submallas,
        );
        let dinamico = (
            ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<VerticeColor>(MAX_CUADROS * 4),
            ),
            ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(
                    &(0..MAX_CUADROS as u16)
                        .flat_map(|q| {
                            let i = q * 4;
                            [i, i + 1, i + 2, i, i + 2, i + 3]
                        })
                        .collect::<Vec<u16>>(),
                ),
            ),
        );
        Self {
            ancho,
            alto,
            mallas: gpu,
            texturas,
            materiales,
            blanca,
            iluminado,
            corteza,
            hojas,
            recorte_opaco,
            recorte_suave,
            nube,
            diamante_atras,
            diamante_frente,
            cielo,
            particulas,
            aditivo,
            terreno_p,
            pasto_p,
            terreno: None,
            tiempo: 0.0,
            cubo,
            dinamico,
        }
    }

    fn material(
        m: &DatosMaterial,
        texturas: &HashMap<String, TextureId>,
        blanca: TextureId,
    ) -> Material3d {
        let tipo = tipo_de(m.shader.as_deref());
        let principal = ["_MainTex", "_FrontTex"]
            .iter()
            .find_map(|n| m.textura(n));
        let textura = principal
            .and_then(|t| t.textura.as_ref())
            .and_then(|t| texturas.get(t).copied())
            .unwrap_or(blanca);
        let st = principal.map_or([1.0, 1.0, 0.0, 0.0], |t| {
            [t.escala[0], t.escala[1], t.desplazamiento[0], t.desplazamiento[1]]
        });
        let brillo = m.shader.as_deref().is_some_and(|s| s.contains("Spec"));
        let especular = if brillo {
            let c = m.color("_SpecColor", [0.5, 0.5, 0.5, 1.0]);
            [c[0], c[1], c[2], m.flotante("_Shininess", 0.078125) * 128.0]
        } else {
            [0.0; 4]
        };
        let cubo = |n: &str| {
            m.textura(n)
                .and_then(|t| t.textura.as_ref())
                .and_then(|t| texturas.get(t).copied())
                .unwrap_or(blanca)
        };
        Material3d {
            tipo,
            textura,
            st,
            color: m.color("_Color", [1.0; 4]),
            especular,
            oclusion: [
                m.flotante("_AO", 2.4),
                m.flotante("_BaseLight", 0.35),
                m.flotante("_Occlusion", 7.5),
                0.0,
            ],
            corte: m.flotante("_Cutoff", 0.5),
            tinte: if tipo == Tipo::Cielo {
                m.color("_Tint", [0.5; 4])
            } else {
                m.color("_TintColor", [0.5; 4])
            },
            cubos: [cubo("_RefractTex"), cubo("_ReflectTex")],
        }
    }

    /// Sube a la GPU el terreno de la escena: la malla alrededor de `centro` (la cámara) hasta
    /// `radio`, y el pasto de los parches a menos de la distancia de los detalles.
    #[allow(clippy::too_many_arguments)]
    pub fn cargar_terreno(
        &mut self,
        terreno: &Terreno,
        parches: &[crate::datos3d::ParcheDetalle],
        centro: Vec3,
        radio: f32,
        distancia: f32,
        densidad: f32,
    ) {
        self.quitar_terreno();
        let ctx = unsafe { macroquad::window::get_internal_gl() }.quad_context;
        let (v, i) = terreno.malla(centro, radio);
        let textura = |n: &Option<String>| {
            n.as_ref()
                .and_then(|n| self.texturas.get(n).copied())
                .unwrap_or(self.blanca)
        };
        let d = &terreno.datos;
        let tamano = [
            d.escala[0] * (d.alturas[0] - 1) as f32,
            d.escala[2] * (d.alturas[1] - 1) as f32,
        ];
        let capa = |k: usize| d.capas.get(k);
        let repetir = |k: usize, e: usize| capa(k).map_or(1.0, |c| tamano[e] / c.tamano[e]);
        let mut pasto = Vec::new();
        for (k, vertices, indices) in terreno.pasto(parches, centro, distancia, densidad) {
            pasto.push((
                ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&vertices),
                ),
                ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&indices),
                ),
                indices.len() as i32,
                textura(&d.detalles.prototipos[k].textura),
            ));
        }
        let gpu = TerrenoGpu {
            vertices: ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice::<VerticeTerreno>(&v),
            ),
            indices: ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&i),
            ),
            n: i.len() as i32,
            control: textura(&d.mezcla.first().cloned().flatten()),
            capas: [
                textura(&capa(0).and_then(|c| c.textura.clone())),
                textura(&capa(1).and_then(|c| c.textura.clone())),
            ],
            repetir: [repetir(0, 0), repetir(0, 1), repetir(1, 0), repetir(1, 1)],
            pasto,
            tinte: d.detalles.tinte,
            fuerza: d.detalles.fuerza,
            cantidad: d.detalles.cantidad,
            rapidez: d.detalles.rapidez,
            distancia,
        };
        self.terreno = Some(gpu);
    }

    /// Al cambiar de escena: libera el terreno.
    pub fn quitar_terreno(&mut self) {
        if let Some(t) = self.terreno.take() {
            let ctx = unsafe { macroquad::window::get_internal_gl() }.quad_context;
            ctx.delete_buffer(t.vertices);
            ctx.delete_buffer(t.indices);
            for (v, i, _, _) in t.pasto {
                ctx.delete_buffer(v);
                ctx.delete_buffer(i);
            }
        }
    }

    pub fn textura(&self, nombre: &str) -> Option<TextureId> {
        self.texturas.get(nombre).copied()
    }

    /// Dibuja la escena desde `camara` en `pase`: borra con el color de fondo o con el skybox
    /// `cielo`, dibuja los MeshRenderer por colas y las partículas. `luz` es la luz puntual de la
    /// escena (todas las escenas de Go Ball tienen una).
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar(
        &mut self,
        pase: RenderPass,
        escena: &Escena3d,
        camara: &Camara3d,
        fondo: Color,
        cielo: Option<&str>,
        ambiente: Color,
        luz: Option<LuzPuntual>,
        particulas: &[Billboards],
    ) {
        let mut gl = unsafe { macroquad::window::get_internal_gl() };
        gl.flush();
        let ctx = gl.quad_context;
        ctx.begin_pass(
            Some(pase),
            PassAction::Clear {
                color: Some((fondo.r, fondo.g, fondo.b, fondo.a)),
                depth: Some(1.0),
                stencil: None,
            },
        );
        let vp = camara.matriz();
        if let Some(m) = cielo.and_then(|c| self.materiales.get(c)) {
            let giro = Mat4::from_scale(vec3(1.0, -1.0, 1.0))
                * camara.proyeccion()
                * Mat4::from_scale(vec3(1.0, 1.0, -1.0))
                * Mat4::from_quat(camara.rotacion.inverse());
            ctx.apply_pipeline(&self.cielo);
            for (b, n) in &self.cubo.1 {
                ctx.apply_bindings(&Bindings {
                    vertex_buffers: vec![self.cubo.0],
                    index_buffer: *b,
                    images: vec![m.textura],
                });
                ctx.apply_uniforms(UniformsSource::table(&USimple {
                    mvp: columnas(giro),
                    st: [1.0, 1.0, 0.0, 0.0],
                    color: m.tinte,
                    recorte: [0.0; 4],
                }));
                ctx.draw(0, *n, 1);
            }
        }
        let luz = luz.unwrap_or(LuzPuntual {
            posicion: Vec3::ZERO,
            color: Vec3::ZERO,
            rango: 1.0,
        });
        let luz_posicion = [luz.posicion.x, luz.posicion.y, luz.posicion.z, 1.0 / luz.rango];
        let luz_color = [luz.color.x, luz.color.y, luz.color.z, 1.0];
        let ambiente = v4(ambiente);
        let camara_mundo = [camara.posicion.x, camara.posicion.y, camara.posicion.z, 1.0];

        // El terreno (cola 1900) y su pasto (2200): el terreno, antes que todo lo opaco, y el pasto
        // después (con el depth test, el orden entre opacos no cambia la imagen).
        if let Some(t) = &self.terreno {
            ctx.apply_pipeline(&self.terreno_p);
            ctx.apply_bindings(&Bindings {
                vertex_buffers: vec![t.vertices],
                index_buffer: t.indices,
                images: vec![t.control, t.capas[0], t.capas[1]],
            });
            ctx.apply_uniforms(UniformsSource::table(&UTerreno {
                mvp: columnas(vp),
                repetir: t.repetir,
                ambiente,
                luz_posicion,
                luz_color,
            }));
            ctx.draw(0, t.n, 1);
            let derecha = camara.rotacion * Vec3::X;
            let arriba = camara.rotacion * Vec3::Y;
            let d2 = t.distancia * t.distancia;
            for (vb, ib, n, tex) in &t.pasto {
                ctx.apply_pipeline(&self.pasto_p);
                ctx.apply_bindings(&Bindings {
                    vertex_buffers: vec![*vb],
                    index_buffer: *ib,
                    images: vec![*tex],
                });
                ctx.apply_uniforms(UniformsSource::table(&UPasto {
                    mvp: columnas(vp),
                    onda: [self.tiempo * t.rapidez, t.fuerza, t.cantidad, d2],
                    camara: [camara.posicion.x, camara.posicion.y, camara.posicion.z, 1.0 / d2],
                    derecha: [derecha.x, derecha.y, derecha.z, 0.0],
                    arriba: [arriba.x, arriba.y, arriba.z, 0.0],
                    tinte: t.tinte,
                    ambiente,
                    luz_posicion,
                    luz_color,
                    corte: [0.5, 0.0, 0.0, 0.0],
                }));
                ctx.draw(0, *n, 1);
            }
        }
        // Lo que hay que dibujar: (cola, distancia, objeto, submalla, material).
        let mut lista: Vec<(i32, f32, Id, usize, &str)> = Vec::new();
        for id in escena.ids() {
            let o = escena.objeto(id);
            let (Some(r), Some(malla)) = (&o.renderer, &o.malla) else {
                continue;
            };
            if !r.activo || !escena.activo_en_jerarquia(id) {
                continue;
            }
            let Some(gpu) = self.mallas.get(malla) else {
                continue;
            };
            let centro = escena.matriz_mundo(id).transform_point3(gpu.centro);
            let distancia = (centro - camara.posicion).length();
            for (i, m) in r.materiales.iter().enumerate().take(gpu.submallas.len()) {
                let Some(m) = m.as_deref() else {
                    continue;
                };
                let Some(mat) = self.materiales.get(m) else {
                    continue;
                };
                if mat.tipo == Tipo::Nada || mat.tipo == Tipo::Particulas {
                    continue;
                }
                lista.push((cola(mat.tipo), distancia, id, i, m));
            }
        }
        // Los opacos de adelante hacia atrás y los transparentes de atrás hacia adelante.
        lista.sort_by(|a, b| {
            a.0.cmp(&b.0).then_with(|| {
                if a.0 >= 3000 {
                    b.1.total_cmp(&a.1)
                } else {
                    a.1.total_cmp(&b.1)
                }
            })
        });
        for (_, _, id, sub, m) in lista {
            let mat = &self.materiales[m];
            let gpu = &self.mallas[escena.objeto(id).malla.as_ref().expect("malla")];
            let modelo = escena.matriz_mundo(id);
            let normales = modelo.inverse().transpose();
            let mvp = vp * modelo;
            let (ib, n) = gpu.submallas[sub];
            let enlazar = |ctx: &mut dyn RenderingBackend, images: Vec<TextureId>| {
                ctx.apply_bindings(&Bindings {
                    vertex_buffers: vec![gpu.vertices],
                    index_buffer: ib,
                    images,
                });
            };
            match mat.tipo {
                Tipo::Iluminado => {
                    ctx.apply_pipeline(&self.iluminado);
                    enlazar(ctx, vec![mat.textura]);
                    ctx.apply_uniforms(UniformsSource::table(&UIluminado {
                        mvp: columnas(mvp),
                        modelo: columnas(modelo),
                        normales: columnas(normales),
                        st: mat.st,
                        color: mat.color,
                        ambiente,
                        luz_posicion,
                        luz_color,
                        especular: mat.especular,
                        camara: camara_mundo,
                    }));
                    ctx.draw(0, n, 1);
                }
                Tipo::Corteza | Tipo::Hojas => {
                    let hojas = mat.tipo == Tipo::Hojas;
                    ctx.apply_pipeline(if hojas { &self.hojas } else { &self.corteza });
                    enlazar(ctx, vec![mat.textura]);
                    ctx.apply_uniforms(UniformsSource::table(&UArbol {
                        mvp: columnas(mvp),
                        modelo: columnas(modelo),
                        color: mat.color,
                        ambiente,
                        luz_posicion,
                        luz_color,
                        oclusion: mat.oclusion,
                        hojas: [if hojas { 1.0 } else { 0.0 }, mat.corte, 0.0, 0.0],
                        corte: [if hojas { 1.0 } else { 0.0 }, mat.corte, 0.0, 0.0],
                    }));
                    ctx.draw(0, n, 1);
                }
                Tipo::Recorte => {
                    for (p, modo) in [(&self.recorte_opaco, 1.0), (&self.recorte_suave, 2.0)] {
                        ctx.apply_pipeline(p);
                        escribir_profundidad(modo < 1.5);
                        enlazar(ctx, vec![mat.textura]);
                        ctx.apply_uniforms(UniformsSource::table(&USimple {
                            mvp: columnas(mvp),
                            st: mat.st,
                            color: mat.color,
                            recorte: [mat.corte, modo, 0.0, 0.0],
                        }));
                        ctx.draw(0, n, 1);
                    }
                    escribir_profundidad(true);
                }
                Tipo::Nube => {
                    ctx.apply_pipeline(&self.nube);
                    escribir_profundidad(false);
                    enlazar(ctx, vec![mat.textura]);
                    ctx.apply_uniforms(UniformsSource::table(&USimple {
                        mvp: columnas(mvp),
                        st: mat.st,
                        color: [1.0; 4],
                        recorte: [0.0; 4],
                    }));
                    ctx.draw(0, n, 1);
                    escribir_profundidad(true);
                }
                Tipo::Diamante => {
                    for (p, pasada) in [(&self.diamante_atras, 0.0), (&self.diamante_frente, 1.0)] {
                        ctx.apply_pipeline(p);
                        escribir_profundidad(pasada > 0.5);
                        enlazar(ctx, mat.cubos.to_vec());
                        ctx.apply_uniforms(UniformsSource::table(&UDiamante {
                            mvp: columnas(mvp),
                            modelo: columnas(modelo),
                            normales: columnas(normales),
                            camara: camara_mundo,
                            color: mat.color,
                            pasada: [pasada, 0.0, 0.0, 0.0],
                        }));
                        ctx.draw(0, n, 1);
                    }
                }
                _ => {}
            }
        }
        // Las partículas van en la cola transparente (aquí, después de las mallas: en las
        // escenas no se cruzan con otros transparentes).
        let derecha = camara.rotacion * Vec3::X;
        let arriba = camara.rotacion * Vec3::Y;
        let escala_pantalla = self.alto / 2.0 / (camara.campo_vision.to_radians() / 2.0).tan();
        for b in particulas {
            let Some(mat) = self.materiales.get(b.material) else {
                continue;
            };
            let mut vertices = Vec::with_capacity(b.particulas.len() * 4);
            for &(p, tamano, rotacion, c) in b.particulas.iter().take(MAX_CUADROS) {
                let profundidad = (p - camara.posicion).dot(camara.rotacion * Vec3::Z);
                if profundidad <= camara.cerca {
                    continue;
                }
                // ParticleRenderer.maxParticleSize: como mucho esa fracción del alto de la
                // pantalla.
                let pixeles = tamano * escala_pantalla / profundidad;
                let tamano = if pixeles > b.tamano_maximo * self.alto {
                    b.tamano_maximo * self.alto * profundidad / escala_pantalla
                } else {
                    tamano
                };
                let (s, co) = rotacion.to_radians().sin_cos();
                let h = tamano / 2.0;
                let ejes = (
                    (derecha * co + arriba * s) * h,
                    (arriba * co - derecha * s) * h,
                );
                for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                    let q = p + ejes.0 * (u * 2.0 - 1.0) + ejes.1 * (v * 2.0 - 1.0);
                    vertices.push(VerticeColor {
                        posicion: q.to_array(),
                        uv: [u, v],
                        color: v4(c),
                    });
                }
            }
            if vertices.is_empty() {
                continue;
            }
            ctx.buffer_update(self.dinamico.0, BufferSource::slice(&vertices));
            ctx.apply_pipeline(&self.particulas);
            escribir_profundidad(false);
            ctx.apply_bindings(&Bindings {
                vertex_buffers: vec![self.dinamico.0],
                index_buffer: self.dinamico.1,
                images: vec![mat.textura],
            });
            let t = mat.tinte;
            ctx.apply_uniforms(UniformsSource::table(&UColor {
                mvp: columnas(vp),
                tinte: [t[0] * 2.0, t[1] * 2.0, t[2] * 2.0, t[3] * 2.0],
            }));
            ctx.draw(0, (vertices.len() / 4 * 6) as i32, 1);
            escribir_profundidad(true);
        }
        ctx.end_render_pass();
    }

    /// Dibuja cuadros aditivos en la pantalla (los destellos de las luces) con una textura.
    pub fn dibujar_pantalla(&mut self, pase: RenderPass, textura: &str, cuadros: &[CuadroPantalla]) {
        let Some(t) = self.texturas.get(textura).copied() else {
            return;
        };
        if cuadros.is_empty() {
            return;
        }
        let mut gl = unsafe { macroquad::window::get_internal_gl() };
        gl.flush();
        let ctx = gl.quad_context;
        ctx.begin_pass(Some(pase), PassAction::Nothing);
        let mut vertices = Vec::new();
        for c in cuadros.iter().take(MAX_CUADROS) {
            let (s, co) = c.angulo.sin_cos();
            let h = c.tamano / 2.0;
            let ex = vec2(co, s) * h;
            let ey = vec2(-s, co) * h;
            let [u0, v0, u1, v1] = c.uv;
            // El borde de arriba del cuadro (b = -1, la y de la pantalla crece hacia abajo) lleva
            // la v más alta.
            for (a, b, u, v) in [
                (-1.0, -1.0, u0, v1),
                (1.0, -1.0, u1, v1),
                (1.0, 1.0, u1, v0),
                (-1.0, 1.0, u0, v0),
            ] {
                let p = c.centro + ex * a + ey * b;
                vertices.push(VerticeColor {
                    posicion: [p.x, p.y, 0.0],
                    uv: [u, v],
                    color: v4(c.color),
                });
            }
        }
        ctx.buffer_update(self.dinamico.0, BufferSource::slice(&vertices));
        ctx.apply_pipeline(&self.aditivo);
        ctx.apply_bindings(&Bindings {
            vertex_buffers: vec![self.dinamico.0],
            index_buffer: self.dinamico.1,
            images: vec![t],
        });
        // Píxeles (origen arriba) a la pantalla del render target (con la y invertida).
        let orto = Mat4::orthographic_rh_gl(0.0, self.ancho, 0.0, self.alto, -1.0, 1.0);
        ctx.apply_uniforms(UniformsSource::table(&UColor {
            mvp: columnas(orto),
            tinte: [1.0; 4],
        }));
        ctx.draw(0, (vertices.len() / 4 * 6) as i32, 1);
        ctx.end_render_pass();
    }
}

/// Una luz puntual de la escena: la primera Light activa.
pub fn luz_de(escena: &Escena3d) -> Option<(Id, LuzPuntual)> {
    escena.ids().into_iter().find_map(|id| {
        let l = escena.objeto(id).luz.as_ref()?;
        (l.activo && escena.activo_en_jerarquia(id)).then(|| {
            (
                id,
                LuzPuntual {
                    posicion: escena.posicion(id),
                    color: vec3(l.color.r, l.color.g, l.color.b) * l.intensidad,
                    rango: l.rango,
                },
            )
        })
    })
}
