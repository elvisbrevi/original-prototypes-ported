// Efectos del juego reescritos en GLSL a partir de sus píxel shaders ps_2_0
// (tools/ps20_desensamblar.py sobre Shaders/negative.xnb y varios.xnb).
// Como en XNA, el efecto sustituye al píxel shader de SpriteBatch y no usa el color del vértice.
//
//   Negative (negative.xnb):           rgb = 1 - rgb, alfa igual.
//   Brightness (varios.xnb, pasada 0): color * Intensity, alfa incluido.
//   TopFade (varios.xnb, pasada 1):    color * v (se desvanece hacia arriba), alfa incluido.
//
// Cada uno existe con las dos mezclas de SpriteBatch: AlphaBlend y Additive.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, UniformType};
use macroquad::prelude::*;

use crate::motor::objeto::Mezcla;
use crate::motor::{Post, Shader};

const VERTICE: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    color = color0 / 255.0;
}
"#;

const SIN_EFECTO: &str = r#"#version 100
precision lowp float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}
"#;

const NEGATIVE: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 c = texture2D(Texture, uv);
    gl_FragColor = vec4(1.0 - c.rgb, c.a);
}
"#;

const BRIGHTNESS: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
uniform sampler2D Texture;
uniform float Intensity;
void main() {
    gl_FragColor = texture2D(Texture, uv) * Intensity;
}
"#;

const TOP_FADE: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, uv) * uv.y;
}
"#;

pub struct Materiales {
    aditivo: Material,
    efectos: [(Shader, [Material; 2]); 3],
}

fn material(fragmento: &str, mezcla: Mezcla, uniformes: Vec<UniformDesc>) -> Material {
    // SpriteBlendMode.AlphaBlend: SourceAlpha / InverseSourceAlpha; Additive: SourceAlpha / One.
    let destino = match mezcla {
        Mezcla::Alfa => BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        Mezcla::Aditiva => BlendFactor::One,
    };
    load_material(
        ShaderSource::Glsl {
            vertex: VERTICE,
            fragment: fragmento,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    destino,
                )),
                ..Default::default()
            },
            uniforms: uniformes,
            ..Default::default()
        },
    )
    .expect("material")
}

impl Materiales {
    pub fn cargar() -> Self {
        let par = |fragmento: &str, uniformes: Vec<UniformDesc>| {
            [Mezcla::Alfa, Mezcla::Aditiva].map(|m| material(fragmento, m, uniformes.clone()))
        };
        Self {
            aditivo: material(SIN_EFECTO, Mezcla::Aditiva, vec![]),
            efectos: [
                (Shader::Negative, par(NEGATIVE, vec![])),
                (
                    Shader::Brightness,
                    par(
                        BRIGHTNESS,
                        vec![UniformDesc::new("Intensity", UniformType::Float1)],
                    ),
                ),
                (Shader::TopFade, par(TOP_FADE, vec![])),
            ],
        }
    }

    /// Fija la mezcla y el efecto de un SpriteBatch.Begin (seguido de efecto.Begin/pass.Begin).
    pub fn usar(&self, mezcla: Mezcla, efecto: Option<Post>) {
        let indice = (mezcla == Mezcla::Aditiva) as usize;
        match efecto {
            None if mezcla == Mezcla::Alfa => gl_use_default_material(),
            None => gl_use_material(&self.aditivo),
            Some(post) => {
                let (_, materiales) = self
                    .efectos
                    .iter()
                    .find(|(s, _)| *s == post.tipo)
                    .expect("efecto cargado");
                let m = &materiales[indice];
                if post.tipo == Shader::Brightness {
                    m.set_uniform("Intensity", post.intensidad);
                }
                gl_use_material(m);
            }
        }
    }
}
