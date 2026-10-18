// Contenido extraído de los .xnb originales con tools/extraer.sh, embebido en el binario. Los
// objetos de los mapas piden sus texturas, fuentes y sonidos por nombre, como Content.Load.

use std::collections::HashMap;

use macroquad::prelude::*;
use xna::SpriteFont;

macro_rules! tabla {
    ($($nombre:literal),* $(,)?) => {
        [$(($nombre, include_bytes!(concat!("../assets/", $nombre)).as_slice())),*]
    };
}

pub struct Assets {
    texturas: HashMap<&'static str, Texture2D>,
    fuentes: HashMap<&'static str, SpriteFont>,
    sonidos: HashMap<&'static str, &'static [u8]>,
    pub mapas: HashMap<&'static str, &'static str>,
}

impl Assets {
    pub fn cargar() -> Self {
        let graficos = tabla!(
            "Graficos/arbusto.png",
            "Graficos/bala.png",
            "Graficos/barril.png",
            "Graficos/bloque4.png",
            "Graficos/bola.png",
            "Graficos/cañon.png",
            "Graficos/cielo.png",
            "Graficos/edificio1.png",
            "Graficos/energia.png",
            "Graficos/fachada1.png",
            "Graficos/fachada2.png",
            "Graficos/fire.png",
            "Graficos/hunter.png",
            "Graficos/kamikaze.png",
            "Graficos/mapa.png",
            "Graficos/maquina.png",
            "Graficos/marco.png",
            "Graficos/mira.png",
            "Graficos/montaña.png",
            "Graficos/nube.png",
            "Graficos/pajaro.png",
            "Graficos/pila.png",
            "Graficos/pixel.png",
            "Graficos/plataforma1.png",
            "Graficos/plataforma2.png",
            "Graficos/set.png",
            "Graficos/set2.png",
            "Graficos/set3.png",
            "Graficos/set4.png",
            "Graficos/star.png",
            "Graficos/subterraneo.png",
            "Graficos/tubo.png",
            "Graficos/yerko.png",
        );
        let texturas = graficos
            .into_iter()
            .map(|(ruta, datos)| {
                let nombre = &ruta["Graficos/".len()..ruta.len() - ".png".len()];
                (nombre, Texture2D::from_file_with_format(datos, None))
            })
            .collect();
        let fuente = |nombre: &'static str, png: &[u8], txt: &str| {
            let textura = Texture2D::from_file_with_format(png, None);
            (nombre, SpriteFont::nueva(textura, txt))
        };
        let fuentes = [
            fuente(
                "normal",
                include_bytes!("../assets/Fuentes/normal.png"),
                include_str!("../assets/Fuentes/normal.txt"),
            ),
            fuente(
                "pixel1",
                include_bytes!("../assets/Fuentes/pixel1.png"),
                include_str!("../assets/Fuentes/pixel1.txt"),
            ),
            fuente(
                "pixel2",
                include_bytes!("../assets/Fuentes/pixel2.png"),
                include_str!("../assets/Fuentes/pixel2.txt"),
            ),
        ]
        .into_iter()
        .collect();
        let sonidos = tabla!(
            "Audios/cancion1.wav",
            "Audios/cancion2.wav",
            "Audios/charge.wav",
            "Audios/crow.wav",
            "Audios/dead.wav",
            "Audios/electricity.wav",
            "Audios/explosion1.wav",
            "Audios/explosion2.wav",
            "Audios/hunter dead.wav",
            "Audios/hunter move.wav",
            "Audios/jump.wav",
            "Audios/kamikaze dead.wav",
            "Audios/kamikaze move.wav",
            "Audios/paper.wav",
            "Audios/shot.wav",
            "Audios/vortice.wav",
            "Audios/walk.wav",
        )
        .into_iter()
        .map(|(ruta, datos)| (&ruta["Audios/".len()..ruta.len() - ".wav".len()], datos))
        .collect();
        let mapas = [
            ("titulo", include_str!("../assets/Mapas/titulo.xml")),
            ("nivel1", include_str!("../assets/Mapas/nivel1.xml")),
            ("nivel2", include_str!("../assets/Mapas/nivel2.xml")),
            ("nivel3", include_str!("../assets/Mapas/nivel3.xml")),
            ("nivel4", include_str!("../assets/Mapas/nivel4.xml")),
        ]
        .into_iter()
        .collect();
        Self {
            texturas,
            fuentes,
            sonidos,
            mapas,
        }
    }

    /// Content.Load<Texture2D>("Graficos/" + nombre).
    pub fn textura(&self, nombre: &str) -> &Texture2D {
        self.texturas
            .get(nombre)
            .unwrap_or_else(|| panic!("falta Graficos/{nombre}"))
    }

    /// Content.Load<SpriteFont>("Fuentes/" + nombre).
    pub fn fuente(&self, nombre: &str) -> &SpriteFont {
        self.fuentes
            .get(nombre)
            .unwrap_or_else(|| panic!("falta Fuentes/{nombre}"))
    }

    /// Los bytes WAV de "Audios/" + nombre.
    pub fn sonido(&self, nombre: &str) -> &'static [u8] {
        self.sonidos
            .get(nombre)
            .unwrap_or_else(|| panic!("falta Audios/{nombre}"))
    }
}
