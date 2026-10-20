// IMGUI de Unity (OnGUI): GUIStyle.Draw, GUI.Label, GUI.Button y GUI.Window, en píxeles de la
// pantalla con el origen arriba a la izquierda. OnGUI se llama una vez por evento del frame
// (MouseDown, MouseUp y al final Repaint, que es cuando se dibuja). Un botón se aprieta con
// MouseDown (pasa a ser el hotControl) y responde con MouseUp dentro de su rectángulo. Cada
// estado del estilo (normal, hover, active) tiene su fondo: si el del estado no tiene, no se
// dibuja ninguno. Las ventanas se dibujan encima del resto y recortan su contenido.

use std::collections::HashMap;

use macroquad::prelude::*;

use crate::datos::{EstadoEstilo, Estilo};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Evento {
    MouseDown,
    MouseUp,
    Repaint,
}

/// Una fuente dinámica con las métricas verticales de FreeType (en fracciones del tamaño).
pub struct FuenteGui {
    pub normal: Font,
    /// La variante para FontStyle.Bold; si no hay, la normal.
    pub negrita: Option<Font>,
    pub ascenso: f32,
    pub alto_linea: f32,
}

enum Comando {
    Textura {
        textura: Texture2D,
        destino: Rect,
        uv: Rect,
        color: Color,
    },
    Texto {
        texto: String,
        fuente: String,
        negrita: bool,
        tamano: u16,
        x: f32,
        y: f32,
        color: Color,
    },
}

pub struct Gui {
    fuentes: HashMap<String, FuenteGui>,
    texturas: HashMap<String, Texture2D>,
    pub evento: Evento,
    /// Event.mousePosition; None si no hay puntero (un toque que ya terminó).
    pub raton: Option<Vec2>,
    /// GUIUtility.hotControl.
    caliente: i32,
    siguiente_id: i32,
    usado: bool,
    origen: Vec2,
    recorte: Option<Rect>,
    /// 0: la GUI de los scripts; 1: las ventanas, que van encima.
    capa: usize,
    comandos: [Vec<Comando>; 2],
}

impl Gui {
    pub fn nueva(
        fuentes: HashMap<String, FuenteGui>,
        texturas: HashMap<String, Texture2D>,
    ) -> Self {
        Self {
            fuentes,
            texturas,
            evento: Evento::Repaint,
            raton: None,
            caliente: 0,
            siguiente_id: 1,
            usado: false,
            origen: Vec2::ZERO,
            recorte: None,
            capa: 0,
            comandos: [Vec::new(), Vec::new()],
        }
    }

    /// Los eventos del frame, en orden: los del botón del ratón y al final Repaint.
    pub fn eventos(pulsado: bool, soltado: bool) -> Vec<Evento> {
        let mut e = Vec::new();
        if pulsado {
            e.push(Evento::MouseDown);
        }
        if soltado {
            e.push(Evento::MouseUp);
        }
        e.push(Evento::Repaint);
        e
    }

    /// Empieza una llamada a OnGUI para `evento`.
    pub fn empezar(&mut self, evento: Evento) {
        self.evento = evento;
        self.siguiente_id = 1;
        self.usado = false;
        self.origen = Vec2::ZERO;
        self.recorte = None;
        self.capa = 0;
    }

    fn nuevo_id(&mut self) -> i32 {
        let id = self.siguiente_id;
        self.siguiente_id += 1;
        id
    }

    fn absoluto(&self, r: Rect) -> Rect {
        Rect::new(r.x + self.origen.x, r.y + self.origen.y, r.w, r.h)
    }

    fn contiene(&self, r: Rect) -> bool {
        self.raton.is_some_and(|m| r.contains(m))
    }

    /// GUI.Label.
    pub fn label(&mut self, rect: Rect, texto: &str, estilo: &Estilo) {
        if self.evento == Evento::Repaint {
            let abs = self.absoluto(rect);
            self.dibujar_estilo(abs, texto, estilo, &estilo.normal);
        }
    }

    /// GUI.Button: true en el MouseUp que suelta el botón apretado dentro de él.
    pub fn button(&mut self, rect: Rect, texto: &str, estilo: &Estilo) -> bool {
        let id = self.nuevo_id();
        let abs = self.absoluto(rect);
        match self.evento {
            Evento::MouseDown => {
                if !self.usado && self.contiene(abs) {
                    self.caliente = id;
                    self.usado = true;
                }
                false
            }
            Evento::MouseUp => {
                if self.caliente == id {
                    self.caliente = 0;
                    self.usado = true;
                    return self.contiene(abs);
                }
                false
            }
            Evento::Repaint => {
                let hover = self.contiene(abs) && (self.caliente == 0 || self.caliente == id);
                let estado = if hover && self.caliente == id {
                    &estilo.active
                } else if hover {
                    &estilo.hover
                } else {
                    &estilo.normal
                };
                self.dibujar_estilo(abs, texto, estilo, estado);
                false
            }
        }
    }

    /// GUI.Window: el fondo del estilo y el contenido, con coordenadas relativas a la ventana y
    /// recortado por ella. Devuelve el rectángulo (no se puede arrastrar).
    pub fn ventana(
        &mut self,
        rect: Rect,
        estilo: &Estilo,
        contenido: impl FnOnce(&mut Gui),
    ) -> Rect {
        let _id = self.nuevo_id();
        let (capa, origen, recorte, siguiente) =
            (self.capa, self.origen, self.recorte, self.siguiente_id);
        self.capa = 1;
        if self.evento == Evento::Repaint {
            self.dibujar_estilo(rect, "", estilo, &estilo.normal);
        }
        self.origen = rect.point();
        self.recorte = Some(rect);
        self.siguiente_id = 1000;
        contenido(self);
        self.capa = capa;
        self.origen = origen;
        self.recorte = recorte;
        self.siguiente_id = siguiente;
        rect
    }

    /// GUIStyle.Draw: el fondo del estado (con sus bordes) y el texto alineado en el área sin
    /// el relleno, movida por contentOffset.
    fn dibujar_estilo(&mut self, rect: Rect, texto: &str, estilo: &Estilo, estado: &EstadoEstilo) {
        if let Some(t) = estado
            .fondo
            .as_ref()
            .and_then(|f| self.texturas.get(f))
            .cloned()
        {
            self.fondo(&t, rect, estilo.borde);
        }
        if texto.is_empty() {
            return;
        }
        let Some(nombre) = estilo.fuente.clone() else {
            return;
        };
        let Some(fuente) = self.fuentes.get(&nombre) else {
            return;
        };
        let negrita = estilo.estilo & 1 != 0;
        let tamano = if estilo.tamano == 0 {
            16
        } else {
            estilo.tamano
        } as u16;
        let font = if negrita {
            fuente.negrita.as_ref().unwrap_or(&fuente.normal)
        } else {
            &fuente.normal
        };
        let ancho = measure_text(texto, Some(font), tamano, 1.0).width;
        let alto = fuente.alto_linea * tamano as f32;
        let [izq, der, arr, aba] = estilo.relleno;
        let area = Rect::new(
            rect.x + izq,
            rect.y + arr,
            rect.w - izq - der,
            rect.h - arr - aba,
        );
        let fx = [0.0, 0.5, 1.0][estilo.alineacion as usize % 3];
        let fy = [0.0, 0.5, 1.0][estilo.alineacion as usize / 3];
        let x = area.x + (area.w - ancho) * fx + estilo.desplazamiento[0];
        let y = area.y
            + (area.h - alto) * fy
            + estilo.desplazamiento[1]
            + fuente.ascenso * tamano as f32;
        self.comandos[self.capa].push(Comando::Texto {
            texto: texto.to_owned(),
            fuente: nombre,
            negrita,
            tamano,
            x: x.round(),
            y: y.round(),
            color: Color::new(
                estado.color[0],
                estado.color[1],
                estado.color[2],
                estado.color[3],
            ),
        });
    }

    /// El fondo estirado en el rectángulo, con los bordes sin estirar (nueve partes).
    fn fondo(&mut self, t: &Texture2D, rect: Rect, borde: [f32; 4]) {
        let (tw, th) = (t.width(), t.height());
        let [l, r, a, b] = borde;
        let xs = [rect.x, rect.x + l, rect.x + rect.w - r, rect.x + rect.w];
        let ys = [rect.y, rect.y + a, rect.y + rect.h - b, rect.y + rect.h];
        let us = [0.0, l / tw, 1.0 - r / tw, 1.0];
        let vs = [0.0, a / th, 1.0 - b / th, 1.0];
        for i in 0..3 {
            for j in 0..3 {
                let destino = Rect::new(xs[i], ys[j], xs[i + 1] - xs[i], ys[j + 1] - ys[j]);
                if destino.w <= 0.0 || destino.h <= 0.0 {
                    continue;
                }
                let uv = Rect::new(us[i], vs[j], us[i + 1] - us[i], vs[j + 1] - vs[j]);
                self.textura(t, destino, uv);
            }
        }
    }

    fn textura(&mut self, t: &Texture2D, mut destino: Rect, mut uv: Rect) {
        if let Some(c) = self.recorte {
            let Some(r) = destino.intersect(c) else {
                return;
            };
            let (su, sv) = (uv.w / destino.w, uv.h / destino.h);
            uv = Rect::new(
                uv.x + (r.x - destino.x) * su,
                uv.y + (r.y - destino.y) * sv,
                r.w * su,
                r.h * sv,
            );
            destino = r;
        }
        self.comandos[self.capa].push(Comando::Textura {
            textura: t.clone(),
            destino,
            uv,
            color: WHITE,
        });
    }

    /// Dibuja lo que dejó el Repaint: primero la GUI de los scripts y encima las ventanas.
    pub fn presentar(&mut self) {
        for capa in 0..2 {
            for c in std::mem::take(&mut self.comandos[capa]) {
                match c {
                    Comando::Textura {
                        textura,
                        destino,
                        uv,
                        color,
                    } => {
                        let (tw, th) = (textura.width(), textura.height());
                        draw_texture_ex(
                            &textura,
                            destino.x,
                            destino.y,
                            color,
                            DrawTextureParams {
                                dest_size: Some(destino.size()),
                                source: Some(Rect::new(uv.x * tw, uv.y * th, uv.w * tw, uv.h * th)),
                                ..Default::default()
                            },
                        );
                    }
                    Comando::Texto {
                        texto,
                        fuente,
                        negrita,
                        tamano,
                        x,
                        y,
                        color,
                    } => {
                        let f = &self.fuentes[&fuente];
                        let font = if negrita {
                            f.negrita.as_ref().unwrap_or(&f.normal)
                        } else {
                            &f.normal
                        };
                        draw_text_ex(
                            &texto,
                            x,
                            y,
                            TextParams {
                                font: Some(font),
                                font_size: tamano,
                                color,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }
}
