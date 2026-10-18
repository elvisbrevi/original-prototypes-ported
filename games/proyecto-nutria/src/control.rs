// Port de Proyecto_Nutria.Teclado (Control): "X_" es pulsado en este paso, "XHold" mantenido.
// El mando DirectInput (JoystickGenerico) no se portó: sin mando, el original crea un solo
// control, el teclado, que maneja al primer personaje.

use macroquad::prelude::*;
use museo::Pulsados;

#[derive(Default)]
pub struct Control {
    pub a_: bool,
    pub b_: bool,
    pub start_: bool,
    pub up_: bool,
    pub down_: bool,
    pub b_hold: bool,
    pub left_hold: bool,
    pub right_hold: bool,
    anterior: [bool; 7],
}

impl Control {
    /// Teclado.actualizar, una vez por paso. Los botones táctiles cuentan igual que sus teclas.
    /// (L y R, teclas A y S, se leían pero el juego no los usa.)
    pub fn actualizar(&mut self, tactil: &Pulsados) {
        let actual = [
            is_key_down(KeyCode::Z) || tactil.botones[0],
            is_key_down(KeyCode::X) || tactil.botones[1],
            is_key_down(KeyCode::Enter) || is_key_down(KeyCode::KpEnter) || tactil.start,
            is_key_down(KeyCode::Up) || tactil.arriba,
            is_key_down(KeyCode::Down) || tactil.abajo,
            is_key_down(KeyCode::Left) || tactil.izquierda,
            is_key_down(KeyCode::Right) || tactil.derecha,
        ];
        let pulsada = |i: usize| actual[i] && !self.anterior[i];
        self.a_ = pulsada(0);
        self.b_ = pulsada(1);
        self.start_ = pulsada(2);
        self.up_ = pulsada(3);
        self.down_ = pulsada(4);
        self.b_hold = actual[1];
        self.left_hold = actual[5];
        self.right_hold = actual[6];
        self.anterior = actual;
    }
}
