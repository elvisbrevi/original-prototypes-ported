//! Piezas comunes de los ports del museo, sacadas del port de IMAN:
//!
//! - [`Pantalla`]: el juego se dibuja en una pantalla virtual del tamaño original y se escala a la
//!   ventana con bandas negras.
//! - [`PasoFijo`]: la lógica avanza a pasos fijos, como `Game` de XNA con `IsFixedTimeStep`.
//! - [`Tactil`]: controles táctiles configurables por juego (cruceta, 1–4 botones, START y zonas de
//!   toque) que aparecen solos en móviles y tablets.
//!
//! La versión web necesita la función `museo_es_tactil` en el `index.html` (ver `web/index.html`).

mod pantalla;
mod paso;
pub mod sensores;
mod tactil;

pub use pantalla::Pantalla;
pub use paso::PasoFijo;
pub use tactil::{Boton, ConfigTactil, Cruceta, MAX_BOTONES, MAX_ZONAS, Pulsados, Tactil};

/// En la web no hay ventana que cerrar: los juegos ignoran Esc y las opciones de salir.
pub const WEB: bool = cfg!(target_arch = "wasm32");

/// Dibuja la pantalla virtual en la ventana y encima los controles táctiles. Va al final del frame.
pub fn presentar(pantalla: &Pantalla, tactil: &Tactil, pulsados: &Pulsados) {
    pantalla.presentar(tactil.juego_arriba());
    tactil.dibujar(pulsados);
}
