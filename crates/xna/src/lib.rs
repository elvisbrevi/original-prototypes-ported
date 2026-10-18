//! Equivalentes de XNA 3.1 (y de .NET 2.0) que comparten los ports de juegos XNA del museo, con la
//! misma semántica que el original: `Rectangle`, `MathHelper.Clamp`, `SpriteBatch.Draw`,
//! `SpriteFont`, `System.Random` y los colores con nombre.

mod random;
mod sprite_font;

use macroquad::prelude::*;

pub use random::Random;
pub use sprite_font::SpriteFont;

/// TargetElapsedTime por defecto de XNA: 166667 ticks de 100 ns (~60 Hz).
pub const TICKS_POR_PASO: i64 = 166_667;
/// gameTime.ElapsedGameTime.TotalSeconds con paso fijo, como double.
pub const SEGUNDOS_POR_PASO_F64: f64 = TICKS_POR_PASO as f64 / 10_000_000.0;
/// (float)gameTime.ElapsedGameTime.TotalSeconds.
pub const SEGUNDOS_POR_PASO: f32 = SEGUNDOS_POR_PASO_F64 as f32;
/// gameTime.ElapsedGameTime.TotalMilliseconds con paso fijo.
pub const MILISEGUNDOS_POR_PASO_F64: f64 = TICKS_POR_PASO as f64 / 10_000.0;

/// Colores con nombre de XNA (no coinciden con los de macroquad).
pub mod colores {
    use macroquad::prelude::Color;

    pub const VERDE: Color = Color::new(0.0, 128.0 / 255.0, 0.0, 1.0);
    pub const AMARILLO: Color = Color::new(1.0, 1.0, 0.0, 1.0);
    pub const ROJO: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const ORO: Color = Color::new(1.0, 215.0 / 255.0, 0.0, 1.0);
    pub const CORNFLOWER_BLUE: Color = Color::new(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
}

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
/// así que si min > max el resultado es min. La física de los juegos depende de esto.
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    let value = if value > max { max } else { value };
    if value < min { min } else { value }
}

/// Conversión (int) de C#: trunca hacia cero.
pub fn int(value: f32) -> i32 {
    value as i32
}

/// SpriteEffects de XNA: voltear en horizontal y/o en vertical.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Efectos {
    pub horizontal: bool,
    pub vertical: bool,
}

impl Efectos {
    pub const NINGUNO: Self = Self {
        horizontal: false,
        vertical: false,
    };
    pub const HORIZONTAL: Self = Self {
        horizontal: true,
        vertical: false,
    };
    pub const VERTICAL: Self = Self {
        horizontal: false,
        vertical: true,
    };

    /// FlipHorizontally si `voltear`, None si no.
    pub const fn horizontal_si(voltear: bool) -> Self {
        Self {
            horizontal: voltear,
            vertical: false,
        }
    }
}

/// SpriteBatch.Draw(textura, posición, fuente, color, rotación, origen, escala, efectos, capa) de
/// XNA 3.1: el `origen` (en píxeles de la fuente) cae en `posicion`, la escala y la rotación se
/// aplican alrededor de él, y voltear solo invierte las coordenadas de textura.
#[allow(clippy::too_many_arguments)]
pub fn dibujar(
    textura: &Texture2D,
    posicion: Vec2,
    fuente: Option<Rectangle>,
    color: Color,
    rotacion: f32,
    origen: Vec2,
    escala: f32,
    efectos: Efectos,
) {
    let fuente = fuente.map(|f| f.as_rect()).unwrap_or(Rect::new(
        0.0,
        0.0,
        textura.width(),
        textura.height(),
    ));
    let esquina = posicion - origen * escala;
    draw_texture_ex(
        textura,
        esquina.x,
        esquina.y,
        color,
        DrawTextureParams {
            source: Some(fuente),
            dest_size: Some(fuente.size() * escala),
            rotation: rotacion,
            pivot: Some(posicion),
            flip_x: efectos.horizontal,
            flip_y: efectos.vertical,
        },
    );
}

/// SpriteBatch.Draw(textura, rectángulo destino, color): estira la textura entera al destino.
pub fn dibujar_en(textura: &Texture2D, destino: Rectangle, color: Color) {
    draw_texture_ex(
        textura,
        destino.x as f32,
        destino.y as f32,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(destino.width as f32, destino.height as f32)),
            ..Default::default()
        },
    );
}
