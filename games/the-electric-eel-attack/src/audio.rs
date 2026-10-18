// Sonidos de XNA (SoundEffect + SoundEffectInstance) sobre macroquad.
//
// Cada objeto del mapa tiene sus propias instancias, que se paran y cambian de volumen por
// separado; en macroquad eso exige un `Sound` por instancia. macroquad tampoco dice si un sonido
// sigue sonando, y el juego lo consulta (SoundState.Playing), así que cada instancia recuerda
// hasta cuándo dura. Play() no hace nada mientras la instancia suena, como en XNA.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use macroquad::audio::{
    PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
};
use macroquad::time::get_time;

use crate::assets::Assets;

pub struct Instancia {
    sonido: Sound,
    duracion: f64,
    volumen: f32,
    hasta: Option<f64>,
}

impl Instancia {
    async fn cargar(datos: &[u8]) -> Self {
        Self {
            sonido: load_sound_from_bytes(datos).await.expect("sonido WAV"),
            duracion: duracion_wav(datos),
            volumen: 1.0,
            hasta: None,
        }
    }

    /// instancia.State == SoundState.Playing.
    pub fn sonando(&self) -> bool {
        self.hasta.is_some_and(|t| get_time() < t)
    }

    /// SoundEffectInstance.Play: empieza desde el principio si no está sonando.
    pub fn play(&mut self) {
        if !self.sonando() {
            play_sound(
                &self.sonido,
                PlaySoundParams {
                    looped: false,
                    volume: self.volumen,
                },
            );
            self.hasta = Some(get_time() + self.duracion);
        }
    }

    /// SoundEffectInstance.Stop.
    pub fn stop(&mut self) {
        if self.sonando() {
            stop_sound(&self.sonido);
        }
        self.hasta = None;
    }

    /// SoundEffectInstance.Volume (se aplica también a lo que ya suena).
    pub fn set_volumen(&mut self, volumen: f32) {
        if volumen != self.volumen {
            self.volumen = volumen;
            if self.sonando() {
                set_sound_volume(&self.sonido, volumen);
            }
        }
    }
}

/// Segundos de audio de un WAV PCM: bytes de datos / bytes por segundo.
fn duracion_wav(datos: &[u8]) -> f64 {
    let u32_en = |i: usize| u32::from_le_bytes(datos[i..i + 4].try_into().unwrap());
    let bytes_por_segundo = u32_en(28);
    let mut i = 12;
    while i + 8 <= datos.len() {
        if &datos[i..i + 4] == b"data" {
            return u32_en(i + 4) as f64 / bytes_por_segundo as f64;
        }
        i += 8 + u32_en(i + 4) as usize;
    }
    0.0
}

/// Engine.Audios.Sonido: un sonido de un objeto, con su volumen del mapa y su instancia.
pub struct Sonido {
    pub nombre: String,
    pub volumen: f32,
    pub activo: bool,
    pub instancia: Rc<RefCell<Instancia>>,
}

impl Sonido {
    pub fn play(&self) {
        if self.activo {
            self.instancia.borrow_mut().play();
        }
    }
}

/// Instancias de sonido cargadas al empezar (cargar un nivel no puede esperar a que el navegador
/// decodifique audio). Hay dos juegos de tantas instancias de cada sonido como el nivel que más
/// usa, y cada carga de nivel usa el juego que no usó la anterior: así los sonidos del nivel que
/// se deja (por ejemplo "dead" al reiniciar) terminan de sonar, como las instancias de XNA.
pub struct Banco {
    instancias: HashMap<String, Vec<Rc<RefCell<Instancia>>>>,
    siguiente: HashMap<String, usize>,
    mitad: usize,
}

impl Banco {
    pub async fn cargar(assets: &Assets, cantidades: &HashMap<String, usize>) -> Self {
        let mut instancias = HashMap::new();
        for (nombre, &cantidad) in cantidades {
            let mut lista = Vec::new();
            for _ in 0..2 * cantidad {
                lista.push(Rc::new(RefCell::new(
                    Instancia::cargar(assets.sonido(nombre)).await,
                )));
            }
            instancias.insert(nombre.clone(), lista);
        }
        Self {
            instancias,
            siguiente: HashMap::new(),
            mitad: 0,
        }
    }

    /// Antes de cargar un nivel.
    pub fn reiniciar(&mut self) {
        self.siguiente.clear();
        self.mitad = 1 - self.mitad;
    }

    /// Sonido.cargar: una instancia nueva para el sonido `nombre`, con su volumen.
    pub fn tomar(&mut self, nombre: &str, volumen: f32) -> Sonido {
        let lista = &self.instancias[nombre];
        let por_mitad = lista.len() / 2;
        let i = self.siguiente.entry(nombre.to_owned()).or_insert(0);
        let instancia = lista[self.mitad * por_mitad + *i % por_mitad].clone();
        *i += 1;
        instancia.borrow_mut().set_volumen(volumen);
        Sonido {
            nombre: nombre.to_owned(),
            volumen,
            activo: false,
            instancia,
        }
    }
}

/// Las dos canciones de Game1. Son largas (400 y 213 s a 8 kHz) y macroquad las decodifica
/// enteras, así que solo se tiene cargada la que se está usando.
pub struct Musica {
    datos: [&'static [u8]; 2],
    volumenes: [f32; 2],
    pub canciones: [Option<Instancia>; 2],
    pub activas: [bool; 2],
}

impl Musica {
    pub fn nueva(assets: &Assets) -> Self {
        Self {
            datos: [assets.sonido("cancion1"), assets.sonido("cancion2")],
            // Game1.LoadContent: musica[0] a 0,8 y musica[1] a 0,3.
            volumenes: [0.8, 0.3],
            canciones: [None, None],
            activas: [false, false],
        }
    }

    /// Carga la canción `i` si hace falta y libera la otra.
    pub async fn preparar(&mut self, i: usize) {
        if self.canciones[i].is_none() {
            if let Some(mut otra) = self.canciones[1 - i].take() {
                otra.stop();
            }
            let mut cancion = Instancia::cargar(self.datos[i]).await;
            cancion.set_volumen(self.volumenes[i]);
            self.canciones[i] = Some(cancion);
        }
    }

    pub fn sonando(&self, i: usize) -> bool {
        self.canciones[i].as_ref().is_some_and(Instancia::sonando)
    }

    /// Sonido.stop sobre una canción.
    pub fn stop(&mut self, i: usize) {
        if let Some(c) = &mut self.canciones[i]
            && c.sonando()
        {
            c.stop();
            self.activas[i] = false;
        }
    }

    /// foreach musica: play(); Activo = false.
    pub fn reproducir_activas(&mut self) {
        for i in 0..2 {
            if self.activas[i]
                && let Some(c) = &mut self.canciones[i]
            {
                c.play();
            }
            self.activas[i] = false;
        }
    }
}
