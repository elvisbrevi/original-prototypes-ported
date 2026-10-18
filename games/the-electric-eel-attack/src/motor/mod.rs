// Port de Engine.dll, el motor propio del juego: escenarios cargados de XML, objetos, física,
// cámara (Pantalla), efectos y sonidos.

pub mod escenario;
pub mod fisica;
pub mod objeto;
pub mod shader;

use macroquad::prelude::*;

/// Efecto de un objeto o de la escena (Objeto.Shader). El motor tiene trece, pero los mapas y el
/// juego solo usan estos tres.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Shader {
    #[default]
    Ninguno,
    Negative,
    Brightness,
    TopFade,
}

impl Shader {
    pub fn desde_nombre(nombre: &str) -> Self {
        match nombre {
            "Ninguno" => Self::Ninguno,
            "Negative" => Self::Negative,
            "Brightness" => Self::Brightness,
            "TopFade" => Self::TopFade,
            otro => panic!("efecto {otro} sin portar (ningún mapa original lo usa)"),
        }
    }
}

/// Post_Processing: el efecto y su parámetro (Intensity de Brightness).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Post {
    pub tipo: Shader,
    pub intensidad: f32,
}

impl Post {
    pub fn nuevo(tipo: Shader) -> Option<Self> {
        (tipo != Shader::Ninguno).then_some(Self {
            tipo,
            intensidad: 0.0,
        })
    }
}

/// Engine.Datos.Pantalla: la cámara del motor. Su matriz se guarda y solo se recalcula después de
/// un reset(), así que cambiar la posición sin reset() no mueve la imagen.
pub struct PantallaXna {
    pub posicion: Vec2,
    pub tamano: f32,
    pub inclinacion: f32,
    /// _Width y _Height: el back buffer, 640x480 tras Pantalla.iniciar.
    pub ancho: i32,
    pub alto: i32,
    /// _VWidth y _VHeight: la resolución virtual.
    pub v_ancho: i32,
    pub v_alto: i32,
    matriz: Mat4,
    sucia: bool,
}

impl PantallaXna {
    pub fn reset(&mut self) {
        self.sucia = true;
    }

    /// getTransformationMatrix: Translation(-Posicion) * Scale(Tamaño) * RotationZ(Inclinacion).
    pub fn matriz(&mut self) -> Mat4 {
        if self.sucia {
            self.sucia = false;
            self.matriz = Mat4::from_rotation_z(self.inclinacion)
                * Mat4::from_scale(vec3(self.tamano, self.tamano, 1.0))
                * Mat4::from_translation(vec3(-self.posicion.x, -self.posicion.y, 0.0));
        }
        self.matriz
    }
}

/// Los campos estáticos de Escenario y Pantalla, que comparten todos los niveles.
pub struct Globales {
    pub pausa: bool,
    pub activar_shader: bool,
    pub shader: Shader,
    pub post: Option<Post>,
    /// Escenario.Color_: el color de fondo.
    pub color: [u8; 4],
    pub ancho: i32,
    pub alto: i32,
    pub pantalla: PantallaXna,
}

impl Globales {
    /// Estado tras Game1.LoadContent: Pantalla.iniciar(640, 480) deja el back buffer y la
    /// resolución virtual en 640x480 y la matriz pendiente de calcular.
    pub fn nuevas() -> Self {
        Self {
            pausa: false,
            activar_shader: false,
            shader: Shader::Ninguno,
            post: None,
            color: [255; 4],
            ancho: 640,
            alto: 480,
            pantalla: PantallaXna {
                posicion: Vec2::ZERO,
                tamano: 0.0,
                inclinacion: 0.0,
                ancho: 640,
                alto: 480,
                v_ancho: 640,
                v_alto: 480,
                matriz: Mat4::IDENTITY,
                sucia: true,
            },
        }
    }

    /// Escenario.activarShaders.
    pub fn activar_shaders(&mut self, shader: Shader) {
        self.activar_shader = true;
        match shader {
            Shader::Negative => {
                self.post = Post::nuevo(Shader::Negative);
                self.shader = Shader::Negative;
            }
            Shader::Brightness => {
                self.post = Post::nuevo(Shader::Brightness);
                self.shader = Shader::Brightness;
                if let Some(post) = &mut self.post {
                    post.intensidad = 1.0;
                }
            }
            _ => {}
        }
    }

    /// Escenario.desactivarShaders (deja el PostProcessing como estaba).
    pub fn desactivar_shaders(&mut self) {
        self.activar_shader = false;
        self.shader = Shader::Ninguno;
    }
}
