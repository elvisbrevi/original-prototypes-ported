// Contenido extraído de los .xnb originales con tools/xnb_extract.py, embebido en el binario.

use macroquad::audio::{Sound, load_sound_from_bytes};
use macroquad::prelude::*;

use xna::SpriteFont;

macro_rules! textura {
    ($ruta:literal) => {
        Texture2D::from_file_with_format(include_bytes!(concat!("../assets/", $ruta)), None)
    };
}

pub struct Assets {
    pub nino: Texture2D,
    pub iman: Texture2D,
    pub titulo: Texture2D,
    pub cortina: Texture2D,
    pub ramas: Texture2D,
    pub sketch: Texture2D,
    pub cielos: [Texture2D; 2],
    pub backgrounds: [Texture2D; 2],
    pub suelos: [Texture2D; 2],
    pub fronts: [Texture2D; 2],
    /// Indexado por Diseno: NULL, _01.._04 (_05 existe en el contenido pero ningún escenario la usa).
    pub plataformas: [Texture2D; 5],
    pub fuente: SpriteFont,
    pub salto: Sound,
}

impl Assets {
    pub async fn cargar() -> Self {
        let sketch = textura!("shaders/sketch.png");
        // El sampler SketchSampler del efecto no define filtro: D3D9 usa POINT por defecto.
        sketch.set_filter(FilterMode::Nearest);
        Self {
            nino: textura!("personajes/nino.png"),
            iman: textura!("personajes/iman.png"),
            titulo: textura!("menu/titulo.png"),
            cortina: textura!("menu/cortina.png"),
            ramas: textura!("ramas/01.png"),
            sketch,
            cielos: [textura!("cielos/01.png"), textura!("cielos/02.png")],
            backgrounds: [
                textura!("backgrounds/01.png"),
                textura!("backgrounds/02.png"),
            ],
            suelos: [textura!("suelos/01.png"), textura!("suelos/02.png")],
            fronts: [textura!("fronts/01.png"), textura!("fronts/02.png")],
            plataformas: [
                textura!("plataformas/null.png"),
                textura!("plataformas/01.png"),
                textura!("plataformas/02.png"),
                textura!("plataformas/03.png"),
                textura!("plataformas/04.png"),
            ],
            fuente: SpriteFont::nueva(
                textura!("fuente/fuente.png"),
                include_str!("../assets/fuente/fuente.txt"),
            ),
            salto: load_sound_from_bytes(include_bytes!("../assets/audio/jump.wav"))
                .await
                .expect("jump.wav"),
        }
    }
}
