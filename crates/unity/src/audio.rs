// AudioSource: Play vuelve a empezar el clip (si ya sonaba, lo corta) y Stop lo para. macroquad
// no distingue las reproducciones de un mismo sonido, así que cada clip se trata como una sola
// fuente: en estos juegos cada clip lo usa una fuente a la vez.

use std::collections::HashMap;

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

use crate::escena::FuenteAudio;

pub struct Audio {
    sonidos: HashMap<String, Sound>,
}

impl Audio {
    pub async fn cargar(archivos: &[(&str, &[u8])]) -> Self {
        let mut sonidos = HashMap::new();
        for (nombre, datos) in archivos {
            let sonido = load_sound_from_bytes(datos).await.expect("sonido");
            sonidos.insert((*nombre).to_owned(), sonido);
        }
        Self { sonidos }
    }

    /// AudioSource.Play.
    pub fn play(&self, fuente: &FuenteAudio) {
        if !fuente.activo {
            return;
        }
        let Some(s) = fuente.clip.as_ref().and_then(|c| self.sonidos.get(c)) else {
            return;
        };
        stop_sound(s);
        play_sound(
            s,
            PlaySoundParams {
                looped: fuente.bucle,
                volume: if fuente.mudo { 0.0 } else { fuente.volumen },
            },
        );
    }

    /// AudioSource.PlayOneShot(clip, volumeScale): suena encima de lo que ya suene. En el WebGL
    /// de Unity 5.1 el volumen pasa tal cual a la ganancia de WebAudio, sin recortarlo a 1.
    pub fn play_one_shot(&self, clip: &str, volumen: f32) {
        if let Some(s) = self.sonidos.get(clip) {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: volumen,
                },
            );
        }
    }

    /// AudioSource.Stop.
    pub fn stop(&self, fuente: &FuenteAudio) {
        if let Some(s) = fuente.clip.as_ref().and_then(|c| self.sonidos.get(c)) {
            stop_sound(s);
        }
    }
}
