// Equivalentes mínimos de los tipos de XNA 3.1 que usa el juego original.

use macroquad::prelude::*;

/// TargetElapsedTime por defecto de XNA: 166667 ticks de 100 ns.
pub const TICKS_POR_PASO: i64 = museo::PasoFijo::TICKS_XNA;
/// gameTime.ElapsedGameTime.TotalSeconds con paso fijo.
pub const SEGUNDOS_POR_PASO: f32 = TICKS_POR_PASO as f32 / 10_000_000.0;

pub const ANCHO: f32 = 640.0;
pub const ALTO: f32 = 480.0;

/// Color.Green de XNA (no es el GREEN de macroquad).
pub const VERDE: Color = Color::new(0.0, 128.0 / 255.0, 0.0, 1.0);
pub const AMARILLO: Color = Color::new(1.0, 1.0, 0.0, 1.0);

/// Rectangle de XNA: coordenadas enteras, Right = X + Width, Bottom = Y + Height.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn top(&self) -> i32 {
        self.y
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn intersects(&self, value: &Rectangle) -> bool {
        value.x < self.x + self.width
            && self.x < value.x + value.width
            && value.y < self.y + self.height
            && self.y < value.y + value.height
    }

    pub fn as_rect(&self) -> Rect {
        Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

/// MathHelper.Clamp de XNA 3.1: aplica primero el máximo y después el mínimo,
/// así que si min > max el resultado es min. La física original depende de esto.
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    let value = if value > max { max } else { value };
    if value < min { min } else { value }
}

/// Conversión (int) de C#: trunca hacia cero.
pub fn int(value: f32) -> i32 {
    value as i32
}
