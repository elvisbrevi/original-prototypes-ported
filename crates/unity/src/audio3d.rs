// AudioSource de Unity 3.x con sonido 3D: el volumen baja con la distancia a la AudioListener
// (la cámara) según el rolloff (el logarítmico de FMOD: min / distancia desde la distancia
// mínima; el lineal: de 1 a 0 entre la mínima y la máxima). macroquad no panea, así que el
// sonido no se mueve a izquierda o derecha. Como macroquad no distingue las reproducciones de un
// mismo sonido, cada clip es una sola voz; isPlaying se lleva con la duración del clip.

use std::collections::HashMap;

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};
use macroquad::prelude::Vec3;

use crate::escena3d::FuenteAudio3d;

pub struct Audio3d {
    sonidos: HashMap<String, (Sound, f32)>,
    /// Hasta cuándo suena cada clip (en el reloj real), o None si está en bucle.
    sonando: HashMap<String, Option<f64>>,
}

/// El volumen de una fuente a `distancia` de la cámara.
pub fn volumen_3d(f: &FuenteAudio3d, distancia: f32) -> f32 {
    let base = if f.mudo { 0.0 } else { f.volumen };
    let d = distancia.max(f.distancia_minima);
    let atenuacion = match f.atenuacion {
        1 => {
            let rango = (f.distancia_maxima - f.distancia_minima).max(1e-6);
            (1.0 - (d - f.distancia_minima) / rango).clamp(0.0, 1.0)
        }
        _ => f.distancia_minima / d.min(f.distancia_maxima),
    };
    base * atenuacion
}

impl Audio3d {
    /// `archivos`: (nombre del clip, bytes del archivo, duración).
    pub async fn cargar(archivos: &[(&str, &[u8], f32)]) -> Self {
        let mut sonidos = HashMap::new();
        for (nombre, datos, duracion) in archivos {
            let sonido = load_sound_from_bytes(datos).await.expect("sonido");
            sonidos.insert((*nombre).to_owned(), (sonido, *duracion));
        }
        Self {
            sonidos,
            sonando: HashMap::new(),
        }
    }

    /// AudioSource.Play, con el volumen ya atenuado (o el de la fuente si el clip es 2D).
    pub fn play(&mut self, f: &FuenteAudio3d, volumen: f32, reloj: f64) {
        if !f.activo {
            return;
        }
        let Some(clip) = f.clip.as_ref() else {
            return;
        };
        let Some((s, duracion)) = self.sonidos.get(clip) else {
            return;
        };
        stop_sound(s);
        play_sound(
            s,
            PlaySoundParams {
                looped: f.bucle,
                volume: volumen,
            },
        );
        self.sonando
            .insert(clip.clone(), (!f.bucle).then_some(reloj + *duracion as f64));
    }

    /// AudioSource.Stop.
    pub fn stop(&mut self, f: &FuenteAudio3d) {
        if let Some(clip) = f.clip.as_ref()
            && let Some((s, _)) = self.sonidos.get(clip)
        {
            stop_sound(s);
            self.sonando.remove(clip);
        }
    }

    /// AudioSource.isPlaying.
    pub fn suena(&self, f: &FuenteAudio3d, reloj: f64) -> bool {
        f.clip
            .as_ref()
            .and_then(|c| self.sonando.get(c))
            .is_some_and(|fin| fin.is_none_or(|t| reloj < t))
    }

    /// Al cargar otra escena, Unity destruye sus AudioSource y sus sonidos se cortan.
    pub fn parar_todo(&mut self) {
        for (s, _) in self.sonidos.values() {
            stop_sound(s);
        }
        self.sonando.clear();
    }
}

/// La distancia entre una fuente y la cámara.
pub fn distancia(fuente: Vec3, oyente: Vec3) -> f32 {
    (fuente - oyente).length()
}
