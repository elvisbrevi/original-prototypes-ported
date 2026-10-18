// Pantalla virtual: el juego dibuja a su resolución original y se escala a la ventana con bandas.

use macroquad::prelude::*;

pub struct Pantalla {
    pub ancho: f32,
    pub alto: f32,
    destino: RenderTarget,
    /// Copia sin mezcla: el canal alfa de un render target baja donde se dibujan sprites
    /// semitransparentes, y al presentarlo con mezcla oscurecería esas zonas.
    opaco: Material,
}

/// Cámara con una matriz arbitraria que dibuja en la pantalla virtual.
struct CamaraMatriz {
    matriz: Mat4,
    pase: macroquad::texture::RenderPass,
}

impl Camera for CamaraMatriz {
    fn matrix(&self) -> Mat4 {
        self.matriz
    }

    fn depth_enabled(&self) -> bool {
        false
    }

    fn render_pass(&self) -> Option<macroquad::texture::RenderPass> {
        Some(self.pase.clone())
    }

    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        None
    }
}

const VERTICE: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

const FRAGMENTO: &str = r#"#version 100
precision lowp float;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = vec4(texture2D(Texture, uv).rgb, 1.0);
}
"#;

impl Pantalla {
    /// Pantalla virtual de `ancho` x `alto`, escalada con filtro lineal.
    pub fn nueva(ancho: f32, alto: f32) -> Self {
        let destino = render_target(ancho as u32, alto as u32);
        destino.texture.set_filter(FilterMode::Linear);
        let opaco = load_material(
            ShaderSource::Glsl {
                vertex: VERTICE,
                fragment: FRAGMENTO,
            },
            MaterialParams::default(),
        )
        .expect("material opaco");
        Self {
            ancho,
            alto,
            destino,
            opaco,
        }
    }

    pub fn tamano(&self) -> Vec2 {
        vec2(self.ancho, self.alto)
    }

    pub fn textura(&self) -> &Texture2D {
        &self.destino.texture
    }

    /// Dibuja en la pantalla virtual con `origen` (coordenadas de mundo) en la esquina superior
    /// izquierda y el eje Y hacia abajo.
    pub fn vista(&self, origen: Vec2) {
        set_camera(&Camera2D {
            target: origen + self.tamano() / 2.0,
            zoom: vec2(2.0 / self.ancho, 2.0 / self.alto),
            render_target: Some(self.destino.clone()),
            ..Default::default()
        });
    }

    /// Dibuja en la pantalla virtual con una transformación de mundo a píxeles virtuales (por
    /// ejemplo la matriz de cámara de un SpriteBatch de XNA: traslación, escala y rotación).
    pub fn vista_transformada(&self, transformacion: Mat4) {
        let proyeccion = Mat4::from_scale(vec3(2.0 / self.ancho, 2.0 / self.alto, 1.0))
            * Mat4::from_translation(vec3(-self.ancho / 2.0, -self.alto / 2.0, 0.0));
        set_camera(&CamaraMatriz {
            matriz: proyeccion * transformacion,
            pase: self.destino.render_pass.clone(),
        });
    }

    /// Dónde queda la pantalla virtual en la ventana: escalada sin deformar, centrada, o pegada
    /// arriba si `arriba` (en vertical con controles táctiles, para dejarles el espacio de abajo).
    pub fn destino(&self, arriba: bool) -> Rect {
        let escala = (screen_width() / self.ancho).min(screen_height() / self.alto);
        let tamano = self.tamano() * escala;
        let x = (screen_width() - tamano.x) / 2.0;
        let y = if arriba {
            0.0
        } else {
            (screen_height() - tamano.y) / 2.0
        };
        Rect::new(x, y, tamano.x, tamano.y)
    }

    /// Convierte un punto de la ventana (ratón, toque) a coordenadas de la pantalla virtual.
    pub fn a_virtual(&self, punto: Vec2, arriba: bool) -> Vec2 {
        let d = self.destino(arriba);
        (punto - d.point()) * self.ancho / d.w
    }

    /// Copia `textura` (por ejemplo otro render target) en `destino` sin mezclar, como un
    /// viewport opaco.
    pub fn copiar_opaco(&self, textura: &Texture2D, destino: Rect) {
        gl_use_material(&self.opaco);
        draw_texture_ex(
            textura,
            destino.x,
            destino.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(destino.size()),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// Dibuja la pantalla virtual en la ventana con bandas negras.
    pub fn presentar(&self, arriba: bool) {
        set_default_camera();
        clear_background(BLACK);
        self.copiar_opaco(&self.destino.texture, self.destino(arriba));
    }
}
