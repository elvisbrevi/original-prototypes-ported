// Plugin Audio: cada Play crea una instancia con su etiqueta (o reutiliza la primera del mismo
// archivo que ya no suena). El volumen está en dB (por encima de 0 dB se recorta a volumen 1).
// macroquad no dice si un sonido sigue sonando, así que el final se calcula con la duración del
// Ogg. Detener y cambiar el volumen afectan a todo el archivo: macroquad no distingue las
// reproducciones de un mismo sonido.

use std::collections::HashMap;

use macroquad::audio::{
    PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
};
use macroquad::prelude::get_time;

use crate::sprite::igual;

struct InstanciaAudio {
    archivo: String,
    etiqueta: String,
    fin: f64,
    bucle: bool,
    volumen: f64,
    detenida: bool,
    /// Ya lanzó (o tiene pendiente) su disparador On ended.
    avisada: bool,
}

impl InstanciaAudio {
    fn sonando(&self, ahora: f64) -> bool {
        !self.detenida && (self.bucle || ahora < self.fin)
    }
}

pub struct Audio {
    sonidos: HashMap<String, (Sound, f64)>,
    instancias: Vec<InstanciaAudio>,
    /// Etiquetas de las instancias que terminaron, para el disparador On ended.
    terminadas: Vec<String>,
}

/// dB a volumen lineal, entre 0 y 1.
pub fn db_a_lineal(db: f64) -> f64 {
    10f64.powf(db / 20.0).clamp(0.0, 1.0)
}

/// Volumen lineal a dB.
pub fn lineal_a_db(v: f64) -> f64 {
    v.clamp(0.0, 1.0).ln() / 10f64.ln() * 20.0
}

impl Audio {
    pub async fn cargar(archivos: &[(&str, &[u8])]) -> Self {
        let mut sonidos = HashMap::new();
        for (nombre, datos) in archivos {
            let sonido = load_sound_from_bytes(datos).await.expect("ogg");
            sonidos.insert((*nombre).to_owned(), (sonido, duracion_ogg(datos)));
        }
        Self {
            sonidos,
            instancias: Vec::new(),
            terminadas: Vec::new(),
        }
    }

    /// Las instancias que llegaron a su final desde la última revisión.
    fn revisar(&mut self, ahora: f64) {
        for i in &mut self.instancias {
            if !i.avisada && !i.detenida && !i.bucle && ahora >= i.fin {
                i.avisada = true;
                self.terminadas.push(i.etiqueta.clone());
            }
        }
    }

    /// Play (bucle: la opción "looping").
    pub fn play(&mut self, archivo: &str, bucle: bool, volumen_db: f64, etiqueta: &str) {
        let ahora = get_time();
        self.revisar(ahora);
        let volumen = db_a_lineal(volumen_db);
        let (sonido, duracion) = &self.sonidos[archivo];
        play_sound(
            sonido,
            PlaySoundParams {
                looped: bucle,
                volume: volumen as f32,
            },
        );
        let nueva = InstanciaAudio {
            archivo: archivo.to_owned(),
            etiqueta: etiqueta.to_owned(),
            fin: ahora + duracion,
            bucle,
            volumen,
            detenida: false,
            avisada: false,
        };
        match self
            .instancias
            .iter_mut()
            .find(|i| i.archivo == archivo && !i.sonando(ahora))
        {
            Some(i) => *i = nueva,
            None => self.instancias.push(nueva),
        }
    }

    /// Is tag playing.
    pub fn etiqueta_sonando(&self, etiqueta: &str) -> bool {
        let ahora = get_time();
        self.instancias
            .iter()
            .any(|i| igual(&i.etiqueta, etiqueta) && i.sonando(ahora))
    }

    /// Detiene las instancias elegidas. Con Web Audio, detener una que sonaba también lanza su
    /// evento "ended" y con él el disparador On ended.
    fn detener_si(&mut self, elegir: impl Fn(&InstanciaAudio) -> bool) {
        let ahora = get_time();
        self.revisar(ahora);
        let mut archivos: Vec<String> = Vec::new();
        for i in self.instancias.iter_mut().filter(|i| elegir(i)) {
            if i.sonando(ahora) {
                if !i.avisada {
                    i.avisada = true;
                    self.terminadas.push(i.etiqueta.clone());
                }
                if !archivos.contains(&i.archivo) {
                    archivos.push(i.archivo.clone());
                }
            }
            i.detenida = true;
        }
        for a in archivos {
            stop_sound(&self.sonidos[&a].0);
        }
    }

    /// Stop (por etiqueta).
    pub fn detener(&mut self, etiqueta: &str) {
        self.detener_si(|i| igual(&i.etiqueta, etiqueta));
    }

    /// Stop all.
    pub fn detener_todo(&mut self) {
        self.detener_si(|_| true);
    }

    /// Set volume (por etiqueta, en dB).
    pub fn set_volumen(&mut self, etiqueta: &str, db: f64) {
        let volumen = db_a_lineal(db);
        let mut archivos: Vec<String> = Vec::new();
        for i in self
            .instancias
            .iter_mut()
            .filter(|i| igual(&i.etiqueta, etiqueta))
        {
            i.volumen = volumen;
            if !archivos.contains(&i.archivo) {
                archivos.push(i.archivo.clone());
            }
        }
        for a in archivos {
            set_sound_volume(&self.sonidos[&a].0, volumen as f32);
        }
    }

    /// La expresión Volume: el volumen en dB de la primera instancia con la etiqueta (las que
    /// suenan primero), o 0 si no hay ninguna.
    pub fn volumen(&self, etiqueta: &str) -> f64 {
        let ahora = get_time();
        let con_etiqueta = || {
            self.instancias
                .iter()
                .filter(|i| igual(&i.etiqueta, etiqueta))
        };
        con_etiqueta()
            .find(|i| i.sonando(ahora))
            .or_else(|| con_etiqueta().next())
            .map(|i| lineal_a_db(i.volumen))
            .unwrap_or(0.0)
    }

    /// Las etiquetas de las instancias que terminaron desde la última llamada (disparador On
    /// ended, que el navegador lanza entre ticks).
    pub fn terminados(&mut self) -> Vec<String> {
        self.revisar(get_time());
        std::mem::take(&mut self.terminadas)
    }
}

/// Duración de un Ogg Vorbis: la última posición de granulo entre la frecuencia de muestreo.
fn duracion_ogg(datos: &[u8]) -> f64 {
    let frecuencia = datos
        .windows(7)
        .position(|w| w == b"\x01vorbis")
        .map(|i| u32::from_le_bytes(datos[i + 12..i + 16].try_into().unwrap()))
        .unwrap_or(44100);
    let granulo = datos
        .windows(4)
        .rposition(|w| w == b"OggS")
        .map(|i| u64::from_le_bytes(datos[i + 6..i + 14].try_into().unwrap()))
        .unwrap_or(0);
    granulo as f64 / frecuencia as f64
}
