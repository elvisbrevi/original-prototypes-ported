//! Las piezas del motor de Unity 4.x que usan los ports del museo (juegos 2D con el Web Player),
//! traducidas de su comportamiento documentado y observado, porque el motor es nativo y cerrado:
//!
//! - [`datos`], el proyecto extraído del original con UnityPy (`escena.json`).
//! - [`tiempo`], `Time`, con los `FixedUpdate` a paso fijo y `timeScale`.
//! - [`escena`], GameObjects, Transform, `Instantiate`, `Destroy`, `FindGameObjectsWithTag`.
//! - [`fisica2d`], Rigidbody2D, BoxCollider2D, CircleCollider2D, triggers y
//!   `Physics2D.OverlapCircle`, sobre Box2D ([`box2d`]).
//! - [`animator`], Mecanim con clips de sprites.
//! - [`particulas`], ParticleSystem (Shuriken).
//! - [`render`], la cámara y el orden de dibujo de sprites y partículas.
//! - [`gui`], IMGUI (`OnGUI`, `GUIStyle`, `GUI.Label`, `GUI.Button`, `GUI.Window`).
//! - [`audio`], AudioSource.

pub mod animator;
pub mod audio;
pub mod curvas;
pub mod datos;
pub mod escena;
pub mod fisica2d;
pub mod gui;
pub mod particulas;
pub mod render;
pub mod tiempo;

/// Random.Range(min, max) de floats: entre los dos (también si min > max), con el azar de
/// macroquad.
pub fn azar_entre(min: f32, max: f32) -> f32 {
    min + (max - min) * macroquad::rand::gen_range(0.0, 1.0)
}
