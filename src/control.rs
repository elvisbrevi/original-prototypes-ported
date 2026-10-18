// Port de Iman.Controles.PCInput: "X_" es pulsado en este paso, "x_hold" mantenido.

use macroquad::prelude::*;

#[derive(Default)]
pub struct Control {
    pub a_: bool,
    pub b_: bool,
    pub start_: bool,
    pub up_: bool,
    pub down_: bool,
    pub a_hold: bool,
    pub b_hold: bool,
    pub up_hold: bool,
    pub down_hold: bool,
    pub left_hold: bool,
    pub right_hold: bool,
    anterior: [bool; 7],
}

impl Control {
    /// Se llama una vez por paso fijo, igual que PCInput.actualizar en Game1.Update.
    pub fn actualizar(&mut self) {
        let actual = [
            is_key_down(KeyCode::Z),
            is_key_down(KeyCode::X),
            is_key_down(KeyCode::Enter) || is_key_down(KeyCode::KpEnter),
            is_key_down(KeyCode::Up),
            is_key_down(KeyCode::Down),
            is_key_down(KeyCode::Left),
            is_key_down(KeyCode::Right),
        ];
        let pulsada = |i: usize| actual[i] && !self.anterior[i];

        self.a_ = pulsada(0);
        self.b_ = pulsada(1);
        self.start_ = pulsada(2);
        self.up_ = pulsada(3);
        self.down_ = pulsada(4);
        self.a_hold = actual[0];
        self.b_hold = actual[1];
        self.up_hold = actual[3];
        self.down_hold = actual[4];
        self.left_hold = actual[5];
        self.right_hold = actual[6];
        self.anterior = actual;
    }
}
