// GameTime de XNA con paso fijo: ElapsedGameTime es siempre un paso y TotalGameTime cuenta los
// pasos ya ejecutados (XNA lo incrementa antes de llamar a Update).

use xna::TICKS_POR_PASO;

#[derive(Clone, Copy, Default)]
pub struct GameTime {
    pasos: i64,
}

impl GameTime {
    pub fn avanzar(&mut self) {
        self.pasos += 1;
    }

    /// gameTime.TotalGameTime.TotalSeconds.
    pub fn total_segundos(&self) -> f64 {
        (self.pasos * TICKS_POR_PASO) as f64 / 10_000_000.0
    }
}
