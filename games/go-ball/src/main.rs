// Go Ball — port a macroquad del juego de Unity 3.4 de Elvis Brevi (2011). En curso (PLAN.md,
// 2.9): por ahora carga el proyecto extraído del original y lo resume en pantalla.

mod recursos;

use macroquad::prelude::*;
use unity::datos3d::Proyecto3d;
use unity::malla::Malla;

#[macroquad::main("Go Ball")]
async fn main() {
    let proyecto = Proyecto3d::leer(include_str!("../assets/escena.json"));
    let triangulos: usize = recursos::MALLAS
        .iter()
        .map(|(_, b)| Malla::leer(b).submallas.iter().map(Vec::len).sum::<usize>() / 3)
        .sum();
    let bytes: usize = recursos::TEXTURAS
        .iter()
        .chain(recursos::AUDIO)
        .map(|(_, b)| b.len())
        .sum();
    let resumen = format!(
        "{}: {} escenas, {} triangulos, {} KB de texturas y audio",
        proyecto.producto,
        proyecto.escenas.len(),
        triangulos,
        bytes / 1024
    );
    loop {
        clear_background(BLACK);
        draw_text(&resumen, 20.0, 40.0, 24.0, WHITE);
        next_frame().await
    }
}
