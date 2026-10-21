// Neuro Battle — port a macroquad del prototipo de Unity 5.1 (WebGL con IL2CPP) de Elvis Brevi
// (2015). En curso (PLAN.md, 2.10): por ahora carga el proyecto extraído del original y muestra la
// pantalla del título.

use macroquad::prelude::*;
use unity::datos::Proyecto;

#[macroquad::main("Neuro Battle")]
async fn main() {
    let proyecto = Proyecto::leer(include_str!("../assets/escena.json"));
    let titulo =
        Texture2D::from_file_with_format(include_bytes!("../assets/texturas/splash.png"), None);
    let objetos: usize = proyecto.escenas.iter().map(Vec::len).sum();
    let resumen = format!(
        "{} (Unity {}): {} escenas, {} objetos, {} prefabs, {} sprites",
        proyecto.producto,
        proyecto.version,
        proyecto.escenas.len(),
        objetos,
        proyecto.prefabs.len(),
        proyecto.sprites.len()
    );
    loop {
        clear_background(BLACK);
        draw_texture_ex(
            &titulo,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        draw_text(&resumen, 20.0, screen_height() - 20.0, 20.0, WHITE);
        next_frame().await
    }
}
