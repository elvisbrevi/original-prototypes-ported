// La cámara y el dibujo de los SpriteRenderer y los ParticleSystemRenderer. Como en Unity, la cola
// transparente se ordena por sorting layer, luego por order in layer y luego de lejos a cerca de
// la cámara. La cámara puede ser de perspectiva (Field of View vertical) u ortográfica; sin girar,
// los sprites paralelos a la pantalla se proyectan como rectángulos.

use std::collections::HashMap;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, UniformType};
use macroquad::prelude::*;

use crate::datos::Proyecto;
use crate::escena::{Escena, Id};
use crate::particulas::{Particula, Particulas};

pub struct Camara {
    pub posicion: Vec3,
    pub rotacion: Quat,
    pub fondo: Color,
    pub ortografica: bool,
    pub tamano_ortografico: f32,
    pub campo_vision: f32,
    pub cerca: f32,
    pub ancho: f32,
    pub alto: f32,
}

impl Camara {
    /// La cámara de un objeto con componente Camera, en una pantalla de `ancho` x `alto`.
    pub fn de(escena: &Escena, id: Id, ancho: f32, alto: f32) -> Self {
        let c = escena.objeto(id).camara.as_ref().expect("Camera");
        Self {
            posicion: escena.posicion(id),
            rotacion: escena.rotacion(id),
            fondo: c.fondo,
            ortografica: c.ortografica,
            tamano_ortografico: c.tamano_ortografico,
            campo_vision: c.campo_vision,
            cerca: c.cerca,
            ancho,
            alto,
        }
    }

    /// Píxeles por unidad a una profundidad dada.
    pub fn escala(&self, profundidad: f32) -> f32 {
        if self.ortografica {
            self.alto / 2.0 / self.tamano_ortografico
        } else {
            self.alto / 2.0 / (self.campo_vision.to_radians() / 2.0).tan() / profundidad
        }
    }

    /// Camera.WorldToScreenPoint con el origen arriba a la izquierda: el punto en píxeles y su
    /// profundidad, o nada si queda detrás del plano cercano.
    pub fn proyectar(&self, p: Vec3) -> Option<(Vec2, f32)> {
        let q = self.rotacion.inverse() * (p - self.posicion);
        if !self.ortografica && q.z <= self.cerca {
            return None;
        }
        let e = self.escala(q.z);
        Some((
            vec2(self.ancho / 2.0 + q.x * e, self.alto / 2.0 - q.y * e),
            q.z,
        ))
    }
}

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

/// Particles/Additive y Particles/Alpha Blended: 2 * color del vértice * _TintColor * textura.
const TINTE_POR_DOS: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform vec4 Tinte;
void main() {
    gl_FragColor = 2.0 * color * Tinte * texture2D(Texture, uv);
}
"#;

/// Particles/Multiply: lerp(blanco, color * textura, su alfa), mezclado con Blend Zero SrcColor.
const MULTIPLICAR: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    vec4 previo = color * texture2D(Texture, uv);
    gl_FragColor = mix(vec4(1.0), previo, previo.a);
}
"#;

fn material(fragmento: &str, origen: BlendFactor, destino: BlendFactor) -> Material {
    load_material(
        ShaderSource::Glsl {
            vertex: VERTICE,
            fragment: fragmento,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(Equation::Add, origen, destino)),
                ..Default::default()
            },
            uniforms: vec![UniformDesc::new("Tinte", UniformType::Float4)],
            ..Default::default()
        },
    )
    .expect("material de partículas")
}

pub struct Recursos {
    pub texturas: HashMap<String, Texture2D>,
    /// Los shaders de partículas, por nombre.
    shaders: HashMap<&'static str, Material>,
}

impl Recursos {
    pub fn nuevos(texturas: HashMap<String, Texture2D>) -> Self {
        let alfa = BlendFactor::Value(BlendValue::SourceAlpha);
        let mut shaders = HashMap::new();
        shaders.insert(
            "Particles/Additive",
            material(TINTE_POR_DOS, alfa, BlendFactor::One),
        );
        shaders.insert(
            "Particles/Alpha Blended",
            material(
                TINTE_POR_DOS,
                alfa,
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            ),
        );
        shaders.insert(
            "Particles/Multiply",
            material(
                MULTIPLICAR,
                BlendFactor::Zero,
                BlendFactor::Value(BlendValue::SourceColor),
            ),
        );
        Self { texturas, shaders }
    }
}

fn vertice(p: Vec2, uv: Vec2, c: Color) -> Vertex {
    Vertex::new(p.x, p.y, 0.0, uv.x, uv.y, c)
}

/// Un cuadrilátero con las esquinas en el orden arriba-izquierda, arriba-derecha, abajo-derecha,
/// abajo-izquierda de la textura.
fn cuadrilatero(textura: &Texture2D, esquinas: [Vec2; 4], uv: Rect, color: Color) {
    let uvs = [
        vec2(uv.x, uv.y),
        vec2(uv.x + uv.w, uv.y),
        vec2(uv.x + uv.w, uv.y + uv.h),
        vec2(uv.x, uv.y + uv.h),
    ];
    draw_mesh(&Mesh {
        vertices: (0..4)
            .map(|i| vertice(esquinas[i], uvs[i], color))
            .collect(),
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(textura.clone()),
    });
}

enum Dibujable {
    Sprite(Id),
    Particulas(Id),
}

/// La matriz del Transform sin escala (los emisores de Unity 4 no la usan).
pub fn matriz_sin_escala(escena: &Escena, id: Id) -> Mat4 {
    Mat4::from_rotation_translation(escena.rotacion(id), escena.posicion(id))
}

/// Dibuja la escena vista por `camara`, en píxeles de su pantalla.
pub fn dibujar(escena: &Escena, proyecto: &Proyecto, recursos: &Recursos, camara: &Camara) {
    clear_background(Color::new(
        camara.fondo.r,
        camara.fondo.g,
        camara.fondo.b,
        1.0,
    ));
    let mut cola = Vec::new();
    for id in escena.ids() {
        let o = escena.objeto(id);
        if !escena.activo_en_jerarquia(id) {
            continue;
        }
        let distancia = escena.posicion(id).distance_squared(camara.posicion);
        if let Some(s) = &o.sprite
            && s.activo
            && s.sprite.is_some()
        {
            cola.push((s.capa_orden, s.orden, distancia, Dibujable::Sprite(id)));
        }
        if let Some(r) = o.particulas.as_ref().and_then(|p| p.renderer.as_ref())
            && r.activo
        {
            cola.push((r.capa_orden, r.orden, distancia, Dibujable::Particulas(id)));
        }
    }
    // De lejos a cerca dentro de cada capa y orden; a igual distancia, en el orden de creación.
    cola.sort_by(|a, b| {
        (a.0, a.1)
            .cmp(&(b.0, b.1))
            .then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
    });
    for (_, _, _, d) in cola {
        match d {
            Dibujable::Sprite(id) => dibujar_sprite(escena, proyecto, recursos, camara, id),
            Dibujable::Particulas(id) => dibujar_particulas(escena, proyecto, recursos, camara, id),
        }
    }
}

fn dibujar_sprite(
    escena: &Escena,
    proyecto: &Proyecto,
    recursos: &Recursos,
    camara: &Camara,
    id: Id,
) {
    let s = escena.objeto(id).sprite.as_ref().expect("sprite");
    let Some(datos) = s.sprite.as_ref().and_then(|n| proyecto.sprites.get(n)) else {
        return;
    };
    let Some(textura) = recursos.texturas.get(&datos.textura) else {
        return;
    };
    let m = escena.matriz_mundo(id);
    let [w, h] = datos.tamano;
    let [px, py] = datos.pivote;
    let (x0, x1, y0, y1) = (-px * w, (1.0 - px) * w, -py * h, (1.0 - py) * h);
    let mut esquinas = [Vec2::ZERO; 4];
    for (i, (x, y)) in [(x0, y1), (x1, y1), (x1, y0), (x0, y0)]
        .into_iter()
        .enumerate()
    {
        let Some((p, _)) = camara.proyectar(m.transform_point3(vec3(x, y, 0.0))) else {
            return;
        };
        esquinas[i] = p;
    }
    let (tw, th) = (textura.width(), textura.height());
    let [rx, ry, rw, rh] = datos.rect;
    let uv = Rect::new(rx / tw, ry / th, rw / tw, rh / th);
    cuadrilatero(textura, esquinas, uv, s.color);
}

fn dibujar_particulas(
    escena: &Escena,
    proyecto: &Proyecto,
    recursos: &Recursos,
    camara: &Camara,
    id: Id,
) {
    let sistema: &Particulas = escena.objeto(id).particulas.as_ref().expect("particulas");
    let r = sistema.renderer.as_ref().expect("renderer");
    let Some(mat) = r.material.as_ref().and_then(|m| proyecto.materiales.get(m)) else {
        return;
    };
    let (Some(textura), Some(shader)) = (
        mat.textura.as_ref().and_then(|t| recursos.texturas.get(t)),
        mat.shader.as_deref().and_then(|s| recursos.shaders.get(s)),
    ) else {
        return;
    };
    let tinte = mat.tinte.unwrap_or([0.5; 4]);
    shader.set_uniform("Tinte", tinte);
    gl_use_material(shader);
    let emisor = matriz_sin_escala(escena, id);
    let (x, y) = sistema
        .datos
        .hoja
        .as_ref()
        .map_or((1, 1), |h| (h.x.max(1), h.y.max(1)));
    for p in &sistema.particulas {
        dibujar_particula(sistema, p, &emisor, camara, textura, r, (x, y));
    }
    gl_use_default_material();
}

fn dibujar_particula(
    sistema: &Particulas,
    p: &Particula,
    emisor: &Mat4,
    camara: &Camara,
    textura: &Texture2D,
    r: &crate::particulas::RendererParticulas,
    (columnas, filas): (u32, u32),
) {
    let local = sistema.datos.espacio_local;
    let posicion = if local {
        emisor.transform_point3(p.posicion)
    } else {
        p.posicion
    };
    let Some((centro, profundidad)) = camara.proyectar(posicion) else {
        return;
    };
    let (tamano, color, cuadro) = sistema.aspecto(p);
    let escala = camara.escala(profundidad);
    let maximo = r.tamano_maximo * camara.alto / 2.0;
    let medio = (tamano / 2.0 * escala).min(maximo);
    let (col, fila) = (cuadro % columnas, (cuadro / columnas) % filas);
    let uv = Rect::new(
        col as f32 / columnas as f32,
        fila as f32 / filas as f32,
        1.0 / columnas as f32,
        1.0 / filas as f32,
    );
    let esquinas = if r.modo == 1 {
        // Stretched billboard: estirada a lo largo de la velocidad, centrada en la partícula.
        let v = sistema.velocidad(p, emisor);
        let v = if local {
            emisor.transform_vector3(v)
        } else {
            v
        };
        let rapidez = v.length();
        if rapidez < 1e-6 {
            return;
        }
        let largo = tamano * r.escala_largo + rapidez * r.escala_velocidad;
        let d = v / rapidez * largo / 2.0;
        let (Some((frente, _)), Some((cola, _))) = (
            camara.proyectar(posicion + d),
            camara.proyectar(posicion - d),
        ) else {
            return;
        };
        let eje = frente - cola;
        if eje.length_squared() < 1e-6 {
            return;
        }
        let normal = vec2(-eje.y, eje.x).normalize() * medio;
        // La textura va a lo largo de la velocidad: de la cola (u = 0) al frente (u = 1).
        [
            cola + normal,
            frente + normal,
            frente - normal,
            cola - normal,
        ]
    } else {
        let (s, c) = p.rotacion.sin_cos();
        let giro = |x: f32, y: f32| centro + vec2(x * c - y * s, x * s + y * c) * medio;
        [
            giro(-1.0, -1.0),
            giro(1.0, -1.0),
            giro(1.0, 1.0),
            giro(-1.0, 1.0),
        ]
    };
    cuadrilatero(textura, esquinas, uv, color);
}
