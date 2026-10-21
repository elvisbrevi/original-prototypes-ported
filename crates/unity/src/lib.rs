//! Las piezas del motor de Unity 4.x y 5.x que usan los ports del museo (juegos 2D), traducidas de
//! su comportamiento documentado y observado, porque el motor es nativo y cerrado:
//!
//! - [`datos`], el proyecto extraído del original con UnityPy (`escena.json`).
//! - [`tiempo`], `Time`, con los `FixedUpdate` a paso fijo y `timeScale`.
//! - [`escena`], GameObjects, Transform, `Instantiate`, `Destroy`, `FindGameObjectsWithTag`.
//! - [`fisica2d`], Rigidbody2D (con Interpolate), BoxCollider2D, CircleCollider2D,
//!   PolygonCollider2D (en piezas convexas, [`poligonos`]), triggers, colisiones,
//!   `Physics2D.OverlapCircle` y `OverlapCircleAll`, sobre Box2D ([`box2d`]).
//! - [`animator`], Mecanim con clips de sprites y blend trees 1D.
//! - [`particulas`], ParticleSystem (Shuriken).
//! - [`render`], la cámara y el orden de dibujo de sprites y partículas.
//! - [`gui`], IMGUI (`OnGUI`, `GUIStyle`, `GUI.Label`, `GUI.Button`, `GUI.Window`) y GUIText.
//! - [`matematica`], `Mathf` y `Vector3` (SmoothDamp, MoveTowards).
//! - [`audio`], AudioSource.
//!
//! Y las de los juegos 3D de Unity 3.x:
//!
//! - [`datos3d`], el proyecto extraído (escenas, materiales, mallas, piel, fuentes, terreno).
//! - [`malla`], las mallas con sus submallas y sus triángulos de colisión.

pub mod animator;
pub mod audio;
pub mod curvas;
pub mod datos;
pub mod datos3d;
pub mod escena;
pub mod fisica2d;
pub mod gui;
pub mod malla;
pub mod matematica;
pub mod particulas;
pub mod poligonos;
pub mod render;
pub mod tiempo;

/// Random.Range(min, max) de floats: entre los dos (también si min > max), con el azar de
/// macroquad.
pub fn azar_entre(min: f32, max: f32) -> f32 {
    min + (max - min) * macroquad::rand::gen_range(0.0, 1.0)
}
