// Port de Iman.Shaders.NonPhotoRealistic: técnica "ColorSketch" de PostprocessEffect.fx
// (el efecto del sample Non-Photorealistic de XNA), aplicada al título del menú.

use macroquad::prelude::*;

use xna::TICKS_POR_PASO;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying mediump vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

// Valores por defecto leídos del efecto compilado: SketchThreshold = 0.1, SketchBrightness = 0.333.
const FRAGMENT: &str = r#"#version 100
precision mediump float;
varying mediump vec2 uv;
uniform sampler2D Texture;
uniform sampler2D SketchTexture;
uniform vec2 SketchJitter;
const float SketchThreshold = 0.1;
const float SketchBrightness = 0.333;
void main() {
    vec3 scene = texture2D(Texture, uv).rgb;
    vec3 saturatedScene = clamp((scene - SketchThreshold) * 2.0, 0.0, 1.0);
    // AddressU/V = Wrap en el sampler original.
    vec3 sketchPattern = texture2D(SketchTexture, fract(uv + SketchJitter)).rgb;
    vec3 negativeSketch = (1.0 - saturatedScene) * (1.0 - sketchPattern);
    float sketchResult = dot(1.0 - negativeSketch, vec3(SketchBrightness));
    gl_FragColor = vec4(scene * sketchResult, 1.0);
}
"#;

/// Cada cuánto cambia el desplazamiento aleatorio del patrón (sketchJitterSpeed = 0.05 s).
const TICKS_JITTER: i64 = 500_000;

pub struct NonPhotoRealistic {
    material: Material,
    sketch_jitter: Vec2,
    ticks_hasta_jitter: i64,
}

impl NonPhotoRealistic {
    pub fn nuevo(sketch: &Texture2D) -> Self {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("SketchJitter", UniformType::Float2)],
                textures: vec!["SketchTexture".to_owned()],
                ..Default::default()
            },
        )
        .expect("shader ColorSketch");
        material.set_texture("SketchTexture", sketch.clone());
        Self {
            material,
            sketch_jitter: Vec2::ZERO,
            ticks_hasta_jitter: 0,
        }
    }

    pub fn actualizar(&mut self) {
        self.ticks_hasta_jitter -= TICKS_POR_PASO;
        if self.ticks_hasta_jitter <= 0 {
            self.sketch_jitter = vec2(rand::gen_range(0.0, 1.0), rand::gen_range(0.0, 1.0));
            self.ticks_hasta_jitter += TICKS_JITTER;
        }
    }

    /// Dibuja con el efecto activo (SpriteBlendMode.None: el shader devuelve alfa 1).
    pub fn dibujar(&self, dibujo: impl FnOnce()) {
        self.material
            .set_uniform("SketchJitter", self.sketch_jitter);
        gl_use_material(&self.material);
        dibujo();
        gl_use_default_material();
    }
}
