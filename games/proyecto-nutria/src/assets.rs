// Contenido extraído de los .xnb originales con tools/xnb_extract.py (lista en
// tools/contenido.txt), embebido en el binario.

use macroquad::prelude::*;
use xna::SpriteFont;

macro_rules! textura {
    ($ruta:literal) => {
        Texture2D::from_file_with_format(include_bytes!(concat!("../assets/", $ruta, ".png")), None)
    };
}

pub struct Assets {
    pub marco_coipo: Texture2D,
    pub cielo: Texture2D,
    pub scroll1: Texture2D,
    pub scroll2: Texture2D,
    pub diseno_trasero: Texture2D,
    pub coipo: Texture2D,
    pub pollo: Texture2D,
    pub caja: Texture2D,
    pub moneda: Texture2D,
    pub pistola: Texture2D,
    pub laser: Texture2D,
    pub esfera: Texture2D,
    pub marco_icono: Texture2D,
    /// Graficos/Plataformas/null: también es la barra de vida de los iconos.
    pub plataforma_null: Texture2D,
    pub plataforma_roca: Texture2D,
    pub hoja: Texture2D,
    pub fondo_menu: Texture2D,
    pub fuente: SpriteFont,
}

impl Assets {
    pub fn cargar() -> Self {
        Self {
            marco_coipo: textura!("Graficos/Niveles/Nivel 1/Marco Coipo"),
            cielo: textura!("Graficos/Niveles/Nivel 1/sky"),
            scroll1: textura!("Graficos/Niveles/Nivel 1/scroll1"),
            scroll2: textura!("Graficos/Niveles/Nivel 1/scroll2"),
            diseno_trasero: textura!("Graficos/Niveles/Nivel 1/Plataformas"),
            coipo: textura!("Graficos/Personajes/coipo"),
            pollo: textura!("Graficos/Personajes/pollo"),
            caja: textura!("Graficos/Items/caja"),
            moneda: textura!("Graficos/Items/moneda"),
            pistola: textura!("Graficos/Armas/pistola"),
            laser: textura!("Graficos/Armas/laser"),
            esfera: textura!("Graficos/Iconos/esfera"),
            marco_icono: textura!("Graficos/Iconos/marco"),
            plataforma_null: textura!("Graficos/Plataformas/null"),
            plataforma_roca: textura!("Graficos/Plataformas/roca"),
            hoja: textura!("Graficos/Particulas/hoja"),
            fondo_menu: textura!("Graficos/Menus/fondo"),
            fuente: SpriteFont::nueva(
                textura!("Fuentes/fuentePredeterminada"),
                include_str!("../assets/Fuentes/fuentePredeterminada.txt"),
            ),
        }
    }
}
