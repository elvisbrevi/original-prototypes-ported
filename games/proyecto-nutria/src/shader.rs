// Port de Proyecto_Nutria.Shaders.Blur_Grayscale: el efecto "PauseTechnique" de
// Shaders/blur_grayscale.xnb que se aplica al nivel mientras el menú de pausa está abierto.
// Reescrito en GLSL a partir del píxel shader ps_2_0 del efecto (tools/ps20_desensamblar.py):
// suma 12 muestras alrededor de la coordenada (disco de Poisson, radio 0,005 en coordenadas de
// textura, sin la muestra central), pasa a gris con (r + g + b) / 3, divide por 12 y deja como
// alfa la suma de los 12 alfas. No usa el color del vértice, así que ignora el tinte del sprite.
// Los demás efectos del juego (Filter, Negative) no se usan.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

const VERTICE: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying highp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

// Los offsets son el parámetro "offsets" del efecto (constantes c0..c11), con su valor exacto.
const FRAGMENTO: &str = r#"#version 100
precision highp float;
varying highp vec2 uv;
uniform sampler2D Texture;
vec4 muestra(vec2 offset) {
    return texture2D(Texture, uv + 0.00499999989 * offset);
}
void main() {
    vec4 suma = muestra(vec2(-0.326211989, -0.405804992));
    suma += muestra(vec2(-0.840143979, -0.0735799968));
    suma += muestra(vec2(-0.69591397, 0.457136989));
    suma += muestra(vec2(-0.203345001, 0.620715976));
    suma += muestra(vec2(0.962339997, -0.194983006));
    suma += muestra(vec2(0.473434001, -0.480026007));
    suma += muestra(vec2(0.519456029, 0.767022014));
    suma += muestra(vec2(0.185461, -0.893123984));
    suma += muestra(vec2(0.507430971, 0.0644249991));
    suma += muestra(vec2(0.896420002, 0.412458003));
    suma += muestra(vec2(-0.321940005, -0.932614982));
    suma += muestra(vec2(-0.791558981, -0.597705007));
    float gris = (suma.x + suma.y + suma.z) * 0.333333343;
    gl_FragColor = vec4(vec3(gris * 0.0833333358), min(suma.w, 1.0));
}
"#;

pub struct BlurGrayscale {
    pub activo: bool,
    material: Material,
}

impl BlurGrayscale {
    pub fn cargar() -> Self {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTICE,
                fragment: FRAGMENTO,
            },
            MaterialParams {
                // Mezcla alfa normal de SpriteBatch (SpriteBlendMode.AlphaBlend).
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("shader blur_grayscale");
        Self {
            activo: false,
            material,
        }
    }

    pub fn inicio(&self) {
        if self.activo {
            gl_use_material(&self.material);
        }
    }

    pub fn fin(&self) {
        if self.activo {
            gl_use_default_material();
        }
    }
}
