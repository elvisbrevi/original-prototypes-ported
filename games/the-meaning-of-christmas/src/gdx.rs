// Lo de libGDX 0.9.2 que usa el juego, traducido de su código (decompilado del mismo APK con
// JADX): SpriteBatch.draw con el origen abajo a la izquierda, BitmapFont con la fuente arial-15
// que libGDX trae por defecto, Animation.getKeyFrame, el deltaTime de AndroidGraphics (la media de
// los últimos cinco frames), Gdx.input.justTouched y Music. Las texturas de libGDX filtran con
// Nearest por defecto.

use std::collections::HashMap;

use macroquad::audio::{PlaySoundParams, Sound, play_sound, stop_sound};
use macroquad::prelude::*;

/// Gdx.graphics.getWidth() y getHeight(): un Android HVGA (ver README).
pub const ANCHO: f32 = 320.0;
pub const ALTO: f32 = 480.0;

/// Una textura de libGDX: Texture(FileHandle), con filtro Nearest.
pub fn textura(png: &[u8]) -> Texture2D {
    let t = Texture2D::from_file_with_format(png, None);
    t.set_filter(FilterMode::Nearest);
    t
}

/// SpriteBatch.draw(texture, x, y, width, height) con el color del batch (y hacia arriba).
pub fn dibujar(t: &Texture2D, x: f32, y: f32, ancho: f32, alto: f32, color: Color) {
    draw_texture_ex(
        t,
        x,
        ALTO - y - alto,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(ancho, alto)),
            ..Default::default()
        },
    );
}

/// SpriteBatch.draw(TextureRegion, x, y, width, height): `region` en píxeles de la textura, con
/// el origen arriba a la izquierda.
pub fn dibujar_region(t: &Texture2D, region: Rect, x: f32, y: f32, ancho: f32, alto: f32) {
    draw_texture_ex(
        t,
        x,
        ALTO - y - alto,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(ancho, alto)),
            source: Some(region),
            ..Default::default()
        },
    );
}

struct Glifo {
    src: Rect,
    xoffset: f32,
    /// Ya convertido como BitmapFontData sin voltear: -(alto + yoffset del archivo).
    yoffset: f32,
    xadvance: f32,
    kerning: HashMap<char, f32>,
}

/// BitmapFont de libGDX 0.9.2 leyendo un .fnt de texto de BMFont (BitmapFontData) y dibujando
/// como BitmapFont.draw con `integer` (el constructor por defecto lo pide): `y` es la altura de
/// las mayúsculas, y las posiciones se truncan a enteros.
pub struct BitmapFont {
    textura: Texture2D,
    glifos: HashMap<char, Glifo>,
    ascent: f32,
    pub color: Color,
}

fn campo(linea: &str, nombre: &str) -> i32 {
    linea
        .split(' ')
        .find_map(|p| p.strip_prefix(nombre)?.strip_prefix('='))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

impl BitmapFont {
    pub fn nueva(fnt: &str, png: &[u8]) -> Self {
        let mut lineas = fnt.lines();
        lineas.next();
        let comun = lineas.next().expect("common");
        let base = campo(comun, "base") as f32;
        let mut glifos = HashMap::new();
        let mut kernings = Vec::new();
        for l in lineas {
            if l.starts_with("char ") {
                let id = campo(l, "id") as u32;
                let alto = campo(l, "height") as f32;
                let Some(c) = char::from_u32(id) else {
                    continue;
                };
                glifos.insert(
                    c,
                    Glifo {
                        src: Rect::new(
                            campo(l, "x") as f32,
                            campo(l, "y") as f32,
                            campo(l, "width") as f32,
                            alto,
                        ),
                        xoffset: campo(l, "xoffset") as f32,
                        yoffset: -(alto + campo(l, "yoffset") as f32),
                        xadvance: campo(l, "xadvance") as f32,
                        kerning: HashMap::new(),
                    },
                );
            } else if l.starts_with("kerning ") {
                kernings.push((campo(l, "first"), campo(l, "second"), campo(l, "amount")));
            }
        }
        for (a, b, k) in kernings {
            if let (Some(a), Some(b)) = (char::from_u32(a as u32), char::from_u32(b as u32))
                && let Some(g) = glifos.get_mut(&a)
            {
                g.kerning.insert(b, k as f32);
            }
        }
        // Sin espacio en la fuente, libGDX crea uno sin imagen con el avance de la 'l'.
        if !glifos.contains_key(&' ') {
            let avance = glifos.get(&'l').map_or(0.0, |g| g.xadvance);
            glifos.insert(
                ' ',
                Glifo {
                    src: Rect::default(),
                    xoffset: 0.0,
                    yoffset: 0.0,
                    xadvance: avance,
                    kerning: HashMap::new(),
                },
            );
        }
        // capHeight: el alto de la primera mayúscula de capChars que tenga la fuente.
        let alto_mayusculas = "MNBDCEFKAGHIJLOPQRSTUVWXYZ"
            .chars()
            .find_map(|c| glifos.get(&c))
            .map_or(1.0, |g| g.src.h);
        Self {
            textura: textura(png),
            glifos,
            ascent: base - alto_mayusculas,
            color: WHITE,
        }
    }

    /// BitmapFont.draw(batch, texto, x, y).
    pub fn draw(&self, texto: &str, x: f32, y: f32) {
        let y = (y + self.ascent) as i32 as f32;
        let mut x = x as i32 as f32;
        let mut anterior: Option<&Glifo> = None;
        for c in texto.chars() {
            let Some(g) = self.glifos.get(&c) else {
                continue;
            };
            if let Some(a) = anterior {
                x = (x + a.kerning.get(&c).copied().unwrap_or(0.0)) as i32 as f32;
            }
            if g.src.w > 0.0 {
                draw_texture_ex(
                    &self.textura,
                    x + g.xoffset,
                    ALTO - (y + g.yoffset) - g.src.h,
                    self.color,
                    DrawTextureParams {
                        dest_size: Some(vec2(g.src.w, g.src.h)),
                        source: Some(g.src),
                        ..Default::default()
                    },
                );
            }
            x += g.xadvance;
            anterior = Some(g);
        }
    }
}

/// Gdx.graphics.getDeltaTime() en Android: la media de los últimos cinco frames (WindowedMean)
/// en cuanto hay cinco; antes, el último.
pub struct Reloj {
    valores: [f32; 5],
    ultimo: usize,
    agregados: usize,
    delta: f32,
}

impl Reloj {
    pub fn nuevo() -> Self {
        Self {
            valores: [0.0; 5],
            ultimo: 0,
            agregados: 0,
            delta: 0.0,
        }
    }

    /// Un frame que duró `real` segundos (AndroidGraphics.onDrawFrame).
    pub fn frame(&mut self, real: f32) {
        self.delta = real;
        self.agregados += 1;
        self.valores[self.ultimo] = real;
        self.ultimo = (self.ultimo + 1) % self.valores.len();
    }

    pub fn delta(&self) -> f32 {
        let media = if self.agregados >= self.valores.len() {
            self.valores.iter().sum::<f32>() / self.valores.len() as f32
        } else {
            0.0
        };
        if media == 0.0 { self.delta } else { media }
    }
}

/// Lo que el juego lee de Gdx.input en un frame.
#[derive(Clone, Copy, Default)]
pub struct Entrada {
    /// Gdx.input.justTouched().
    pub tocado: bool,
    /// Gdx.input.getX() y getY(): el último toque, en píxeles con y hacia abajo.
    pub x: f32,
    pub y: f32,
}

/// Music: play y stop. Un Music que llega al final deja de sonar y el juego lo vuelve a empezar
/// en el frame siguiente (`if (!isPlaying()) play()`), así que aquí suena en bucle.
pub struct Musica {
    sonido: Sound,
    sonando: bool,
}

impl Musica {
    pub fn nueva(sonido: Sound) -> Self {
        Self {
            sonido,
            sonando: false,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.sonando
    }

    pub fn play(&mut self) {
        play_sound(
            &self.sonido,
            PlaySoundParams {
                looped: true,
                volume: 1.0,
            },
        );
        self.sonando = true;
    }

    pub fn stop(&mut self) {
        stop_sound(&self.sonido);
        self.sonando = false;
    }
}
