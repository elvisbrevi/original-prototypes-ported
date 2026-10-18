// Pantalla virtual: el juego dibuja a su resolución original y se escala a la ventana con bandas.

use macroquad::prelude::*;

pub struct Pantalla {
    pub ancho: f32,
    pub alto: f32,
    destino: RenderTarget,
}

impl Pantalla {
    /// Pantalla virtual de `ancho` x `alto`, escalada con filtro lineal.
    pub fn nueva(ancho: f32, alto: f32) -> Self {
        let destino = render_target(ancho as u32, alto as u32);
        destino.texture.set_filter(FilterMode::Linear);
        Self {
            ancho,
            alto,
            destino,
        }
    }

    pub fn tamano(&self) -> Vec2 {
        vec2(self.ancho, self.alto)
    }

    pub fn textura(&self) -> &Texture2D {
        &self.destino.texture
    }

    /// Dibuja en la pantalla virtual con `origen` (coordenadas de mundo) en la esquina superior
    /// izquierda y el eje Y hacia abajo.
    pub fn vista(&self, origen: Vec2) {
        set_camera(&Camera2D {
            target: origen + self.tamano() / 2.0,
            zoom: vec2(2.0 / self.ancho, 2.0 / self.alto),
            render_target: Some(self.destino.clone()),
            ..Default::default()
        });
    }

    /// Dónde queda la pantalla virtual en la ventana: escalada sin deformar, centrada, o pegada
    /// arriba si `arriba` (en vertical con controles táctiles, para dejarles el espacio de abajo).
    pub fn destino(&self, arriba: bool) -> Rect {
        let escala = (screen_width() / self.ancho).min(screen_height() / self.alto);
        let tamano = self.tamano() * escala;
        let x = (screen_width() - tamano.x) / 2.0;
        let y = if arriba {
            0.0
        } else {
            (screen_height() - tamano.y) / 2.0
        };
        Rect::new(x, y, tamano.x, tamano.y)
    }

    /// Convierte un punto de la ventana (ratón, toque) a coordenadas de la pantalla virtual.
    pub fn a_virtual(&self, punto: Vec2, arriba: bool) -> Vec2 {
        let d = self.destino(arriba);
        (punto - d.point()) * self.ancho / d.w
    }

    /// Dibuja la pantalla virtual en la ventana con bandas negras.
    pub fn presentar(&self, arriba: bool) {
        set_default_camera();
        clear_background(BLACK);
        let d = self.destino(arriba);
        draw_texture_ex(
            &self.destino.texture,
            d.x,
            d.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(d.size()),
                ..Default::default()
            },
        );
    }
}
