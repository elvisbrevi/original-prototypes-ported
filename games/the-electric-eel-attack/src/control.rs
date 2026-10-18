// Port de EAAE.Control: teclado ("press" es pulsado en este paso, "hold" mantenido). El ratón se
// leía pero ningún nivel lo usa.

use macroquad::prelude::*;
use museo::Pulsados;

#[derive(Default)]
pub struct Control {
    pub reiniciar: bool,
    pub press_z: bool,
    pub press_x: bool,
    pub press_start: bool,
    pub press_esc: bool,
    pub hold_left: bool,
    pub hold_right: bool,
    anterior: [bool; 7],
}

impl Control {
    /// Control.iniciar (y finalizar, que guarda el estado para el paso siguiente). Los botones
    /// táctiles cuentan igual que sus teclas.
    pub fn actualizar(&mut self, tactil: &Pulsados) {
        let actual = [
            is_key_down(KeyCode::R) || tactil.botones[2],
            is_key_down(KeyCode::Z) || tactil.botones[0],
            is_key_down(KeyCode::X) || tactil.botones[1],
            is_key_down(KeyCode::Enter) || is_key_down(KeyCode::KpEnter) || tactil.start,
            is_key_down(KeyCode::Escape),
            is_key_down(KeyCode::Left) || tactil.izquierda,
            is_key_down(KeyCode::Right) || tactil.derecha,
        ];
        let pulsada = |i: usize| actual[i] && !self.anterior[i];
        self.reiniciar = pulsada(0);
        self.press_z = pulsada(1);
        self.press_x = pulsada(2);
        self.press_start = pulsada(3);
        self.press_esc = pulsada(4);
        self.hold_left = actual[5];
        self.hold_right = actual[6];
        self.anterior = actual;
    }
}
