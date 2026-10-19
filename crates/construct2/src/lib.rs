//! Las piezas del runtime de Construct 2 (r2xx, exportaciones HTML5 de 2015) que usan los ports
//! del museo, traducidas del `c2runtime.js` minificado: el proyecto de `data.js`, Sprite, Text,
//! Audio, los comportamientos Platform, Bullet, Turret, Custom Movement, Solid y Wrap, la
//! geometría de colisión, el tiempo del runtime y el estado de las condiciones que lo guardan
//! (Every, Wait, On collision).
//!
//! Las hojas de eventos no se interpretan: cada juego las traduce a mano a Rust (en
//! `games/<juego>/src/juego.rs`) usando estas piezas, con el orden del tick del runtime:
//! dt → quitar destruidas → esperas de Wait → comportamientos → animaciones → cambio de layout →
//! eventos (quitando las destruidas tras cada evento de primer nivel) → olvidar colisiones
//! registradas.

pub mod audio;
pub mod comportamientos;
pub mod datos;
pub mod entrada;
pub mod eventos;
pub mod geometria;
pub mod mundo;
pub mod sprite;
pub mod texto;
pub mod tiempo;

pub use audio::Audio;
pub use datos::{Clase, Proyecto};
pub use entrada::{Entrada, Toques};
pub use eventos::{Cursor, Espera, Esperas, MemoriaColision};
pub use mundo::{Inst, Mundo};
pub use sprite::Sprite;
pub use texto::{Fuentes, Texto};
pub use tiempo::{Cada, dt_del_runtime};
