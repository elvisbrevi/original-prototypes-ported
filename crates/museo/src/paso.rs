// Paso fijo: la lógica avanza siempre en pasos del mismo tamaño, sea cual sea el ritmo de frames.

use macroquad::prelude::get_frame_time;

/// Acumula el tiempo de los frames y dice cuántos pasos de lógica tocan en cada uno.
pub struct PasoFijo {
    segundos: f64,
    acumulado: f64,
}

impl PasoFijo {
    /// TargetElapsedTime por defecto de XNA: 166667 ticks de 100 ns (~60 Hz).
    pub const TICKS_XNA: i64 = 166_667;
    /// Como `Game.Tick` de XNA, no se recuperan más de 500 ms por frame.
    const MAXIMO_POR_FRAME: f64 = 0.5;

    pub fn nuevo(segundos: f64) -> Self {
        Self {
            segundos,
            acumulado: 0.0,
        }
    }

    /// El paso de `Game` de XNA (`IsFixedTimeStep = true`, ~60 Hz).
    pub fn xna() -> Self {
        Self::nuevo(Self::TICKS_XNA as f64 / 10_000_000.0)
    }

    /// Duración de un paso en segundos.
    pub fn segundos(&self) -> f64 {
        self.segundos
    }

    /// Suma el tiempo del último frame y devuelve cuántos pasos hay que ejecutar.
    pub fn pasos(&mut self) -> u32 {
        self.acumulado += (get_frame_time() as f64).min(Self::MAXIMO_POR_FRAME);
        let mut pasos = 0;
        while self.acumulado >= self.segundos {
            self.acumulado -= self.segundos;
            pasos += 1;
        }
        pasos
    }
}
