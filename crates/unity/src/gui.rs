// IMGUI de Unity (OnGUI): GUIStyle.Draw, GUI.Label, GUI.Button y GUI.Window, en píxeles de la
// pantalla con el origen arriba a la izquierda. OnGUI se llama una vez por evento del frame
// (MouseDown, MouseUp y al final Repaint, que es cuando se dibuja). Un botón se aprieta con
// MouseDown (pasa a ser el hotControl) y responde con MouseUp dentro de su rectángulo. Cada
// estado del estilo (normal, hover, active) tiene su fondo: si el del estado no tiene, no se
// dibuja ninguno. Las ventanas se dibujan encima del resto y recortan su contenido; la que tiene el
// foco (la última en la que se apretó el ratón) se dibuja con el estado "on" de su estilo. Aparte,
// texto_gui dibuja los GUIText, que pinta el GUILayer de la cámara antes de OnGUI.
//
// Para Unity 3.x (Go Ball) hay además: fuentes de mapa de bits (la textura con un rectángulo por
// carácter), GUI.color (que tiñe fondos y textos y vuelve a blanco en cada OnGUI), GUI.DrawTexture,
// los tooltips de GUIContent (GUI.tooltip es el del último control bajo el ratón en el Repaint) y,
// con `respaldo`, que un estado sin fondo use el estado normal entero, como el GUIStyle de Unity 3.

use std::collections::HashMap;

use macroquad::prelude::*;

use crate::datos::{EstadoEstilo, Estilo};
use crate::datos3d::FuenteMapa;
use crate::escena::TextoGui;

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
    /// El tamaño de la fuente en Unity (Font.fontSize), el que usa un estilo con tamaño 0.
    pub tamano_defecto: f32,
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
    /// Las fuentes de mapa de bits (Unity 3.x), con su textura.
    fuentes_mapa: HashMap<String, (Texture2D, FuenteMapa)>,
    /// GUI.color.
    pub color: Color,
    /// GUI.tooltip.
    pub tooltip: String,
    /// Un estado sin fondo usa el estado normal (GUIStyle de Unity 3.x).
    pub respaldo: bool,
    /// GUIUtility.mouseUsed: el Repaint dibujó un botón con el ratón encima (lo pone en false
    /// quien lo lee).
    pub raton_sobre_control: bool,
    texturas: HashMap<String, Texture2D>,
    pub evento: Evento,
    /// Event.mousePosition; None si no hay puntero (un toque que ya terminó).
    pub raton: Option<Vec2>,
    /// GUIUtility.hotControl.
    caliente: i32,
    /// La ventana con el foco (su id de GUI.Window).
    enfocada: Option<i32>,
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
            fuentes_mapa: HashMap::new(),
            color: WHITE,
            tooltip: String::new(),
            respaldo: false,
            raton_sobre_control: false,
            texturas,
            evento: Evento::Repaint,
            raton: None,
            caliente: 0,
            enfocada: None,
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

    /// Agrega fuentes de mapa de bits (Unity 3.x) con sus texturas.
    pub fn con_fuentes_mapa(mut self, fuentes: HashMap<String, (Texture2D, FuenteMapa)>) -> Self {
        self.fuentes_mapa = fuentes;
        self
    }

    /// Empieza una llamada a OnGUI para `evento`.
    pub fn empezar(&mut self, evento: Evento) {
        self.evento = evento;
        self.color = WHITE;
        self.tooltip.clear();
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

    /// El estado que se dibuja: el pedido o, con `respaldo`, el normal si el pedido no tiene fondo.
    fn estado<'a>(&self, estilo: &'a Estilo, pedido: &'a EstadoEstilo) -> &'a EstadoEstilo {
        if self.respaldo && pedido.fondo.is_none() {
            &estilo.normal
        } else {
            pedido
        }
    }

    /// El tooltip de un control con el ratón encima (en el Repaint).
    fn mirar_tooltip(&mut self, rect: Rect, tooltip: &str) {
        if self.evento == Evento::Repaint && !tooltip.is_empty() && self.contiene(rect) {
            self.tooltip = tooltip.to_owned();
        }
    }

    /// GUI.DrawTexture(rect, textura): estirada en el rectángulo y teñida con GUI.color.
    pub fn dibujar_textura(&mut self, rect: Rect, textura: &Texture2D) {
        if self.evento == Evento::Repaint {
            let abs = self.absoluto(rect);
            self.textura(textura, abs, Rect::new(0.0, 0.0, 1.0, 1.0));
        }
    }

    /// GUI.Button(rect, new GUIContent(texto, tooltip), estilo).
    pub fn button_con_tooltip(
        &mut self,
        rect: Rect,
        texto: &str,
        tooltip: &str,
        estilo: &Estilo,
    ) -> bool {
        let abs = self.absoluto(rect);
        // GUIStyle.Draw: el tooltip del control con el ratón encima, si no está apretado.
        if self.caliente != self.siguiente_id || !self.contiene(abs) {
            self.mirar_tooltip(abs, tooltip);
        }
        self.button(rect, texto, estilo)
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
                if self.contiene(abs) {
                    self.raton_sobre_control = true;
                }
                let hover = self.contiene(abs) && (self.caliente == 0 || self.caliente == id);
                let estado = if hover && self.caliente == id {
                    &estilo.active
                } else if hover {
                    &estilo.hover
                } else {
                    &estilo.normal
                };
                let estado = self.estado(estilo, estado);
                self.dibujar_estilo(abs, texto, estilo, estado);
                false
            }
        }
    }

    /// GUI.Window(id, rect, función, título): el fondo del estilo con el título y el contenido,
    /// con coordenadas relativas a la ventana y recortado por ella. Apretar el ratón dentro le da
    /// el foco. Devuelve el rectángulo (no se puede arrastrar).
    pub fn ventana(
        &mut self,
        id: i32,
        rect: Rect,
        titulo: &str,
        estilo: &Estilo,
        contenido: impl FnOnce(&mut Gui),
    ) -> Rect {
        let _control = self.nuevo_id();
        let (capa, origen, recorte, siguiente) =
            (self.capa, self.origen, self.recorte, self.siguiente_id);
        self.capa = 1;
        if self.evento == Evento::MouseDown && self.contiene(rect) {
            self.enfocada = Some(id);
        }
        if self.evento == Evento::Repaint {
            let estado = if self.enfocada == Some(id) {
                estilo.on_normal.as_ref().unwrap_or(&estilo.normal)
            } else {
                &estilo.normal
            };
            let estado = self.estado(estilo, estado);
            self.dibujar_estilo(rect, titulo, estilo, estado);
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
        if let Some((textura, mapa)) = self.fuentes_mapa.get(&nombre).cloned() {
            self.texto_mapa(rect, texto, estilo, estado, &textura, &mapa);
            return;
        }
        let Some(fuente) = self.fuentes.get(&nombre) else {
            return;
        };
        let negrita = estilo.estilo & 1 != 0;
        let tamano = if estilo.tamano == 0 {
            fuente.tamano_defecto.round() as u32
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
                estado.color[0] * self.color.r,
                estado.color[1] * self.color.g,
                estado.color[2] * self.color.b,
                estado.color[3] * self.color.a,
            ),
        });
    }

    /// El texto con una fuente de mapa de bits de Unity 3.x: una línea de alto `interlineado`,
    /// alineada en el área sin el relleno; cada carácter se dibuja en su rectángulo `vert`
    /// (relativo al borde de arriba de la línea, con y hacia arriba) y avanza su `ancho`.
    fn texto_mapa(
        &mut self,
        rect: Rect,
        texto: &str,
        estilo: &Estilo,
        estado: &EstadoEstilo,
        textura: &Texture2D,
        mapa: &FuenteMapa,
    ) {
        let caracteres: Vec<_> = texto
            .chars()
            .filter_map(|c| mapa.caracteres.get(&(c as u32).to_string()).copied())
            .collect();
        let ancho: f32 = caracteres.iter().map(|c| c.ancho * mapa.kerning).sum();
        let alto = mapa.interlineado;
        let [izq, der, arr, aba] = estilo.relleno;
        let area = Rect::new(
            rect.x + izq,
            rect.y + arr,
            rect.w - izq - der,
            rect.h - arr - aba,
        );
        let fx = [0.0, 0.5, 1.0][estilo.alineacion as usize % 3];
        let fy = [0.0, 0.5, 1.0][estilo.alineacion as usize / 3];
        let mut x = (area.x + (area.w - ancho) * fx + estilo.desplazamiento[0]).round();
        let y = (area.y + (area.h - alto) * fy + estilo.desplazamiento[1]).round();
        let color = Color::new(
            estado.color[0] * self.color.r,
            estado.color[1] * self.color.g,
            estado.color[2] * self.color.b,
            estado.color[3] * self.color.a,
        );
        let guardado = self.color;
        self.color = color;
        for c in caracteres {
            let [vx, vy, vw, vh] = c.vert;
            let [u, v, uw, vh_uv] = c.uv;
            if vw != 0.0 && vh != 0.0 {
                let destino = Rect::new(x + vx, y - vy, vw, -vh);
                // uv con el origen abajo; la textura (PNG) tiene el origen arriba.
                let uv = Rect::new(u, 1.0 - (v + vh_uv), uw, vh_uv);
                self.textura(textura, destino, uv);
            }
            x += c.ancho * mapa.kerning;
        }
        self.color = guardado;
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
            color: self.color,
        });
    }

    /// Dibuja un GUIText en una pantalla de `ancho` x `alto`: su posición es la del Transform en
    /// coordenadas de la vista (0 a 1, con y hacia arriba) más el desplazamiento en píxeles, y el
    /// anclaje dice qué punto del bloque de texto queda ahí. Cada línea se alinea en el bloque.
    pub fn texto_gui(&self, t: &TextoGui, posicion: Vec2, ancho: f32, alto: f32) {
        if !t.activo || t.texto.is_empty() {
            return;
        }
        let Some(nombre) = t.fuente.as_ref() else {
            return;
        };
        let Some(fuente) = self.fuentes.get(nombre) else {
            return;
        };
        let font = if t.estilo & 1 != 0 {
            fuente.negrita.as_ref().unwrap_or(&fuente.normal)
        } else {
            &fuente.normal
        };
        let tamano = if t.tamano == 0 {
            fuente.tamano_defecto
        } else {
            t.tamano as f32
        };
        let alto_linea = fuente.alto_linea * tamano * t.interlineado;
        let lineas: Vec<&str> = t.texto.lines().collect();
        let anchos: Vec<f32> = lineas
            .iter()
            .map(|l| measure_text(l, Some(font), tamano as u16, 1.0).width)
            .collect();
        let bloque = vec2(
            anchos.iter().copied().fold(0.0, f32::max),
            alto_linea * lineas.len() as f32,
        );
        let punto = vec2(
            posicion.x * ancho + t.desplazamiento.x,
            (1.0 - posicion.y) * alto - t.desplazamiento.y,
        );
        let fx = [0.0, 0.5, 1.0][t.anclaje as usize % 3];
        let fy = [0.0, 0.5, 1.0][t.anclaje as usize / 3];
        let esquina = punto - bloque * vec2(fx, fy);
        let alinear = [0.0, 0.5, 1.0][(t.alineacion as usize).min(2)];
        for (i, (linea, w)) in lineas.iter().zip(&anchos).enumerate() {
            let x = esquina.x + (bloque.x - w) * alinear;
            let y = esquina.y + i as f32 * alto_linea + fuente.ascenso * tamano;
            draw_text_ex(
                linea,
                x.round(),
                y.round(),
                TextParams {
                    font: Some(font),
                    font_size: tamano as u16,
                    color: t.color,
                    ..Default::default()
                },
            );
        }
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
