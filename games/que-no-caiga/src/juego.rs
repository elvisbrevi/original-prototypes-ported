// El tick del runtime de Construct 2 y las dos hojas de eventos del juego ("Start events" y
// "Game events"), traducidas a mano desde data.js (tools/c2_eventos.py; volcado en
// reference/eventos.txt). Los tipos de objeto se llaman t0, t1... porque la exportación
// minificada perdió los nombres; aquí se nombran por su imagen.

use std::collections::HashMap;

use macroquad::prelude::*;

use crate::datos::Proyecto;
use crate::runtime::{Audio, Fuentes, Sprite, Texto};

// Tipos de objeto (índice en data.js).
const PELOTA: usize = 0;
const BOTON_JUGAR: usize = 6;
/// Text con el tiempo ("0"), arriba a la derecha.
const TEXTO_TIEMPO: usize = 7;
/// Text "CLICK PARA EMPEZAR".
const TEXTO_EMPEZAR: usize = 10;
/// Botón "OK" del final.
const REINICIAR: usize = 11;
/// Text con el tiempo en el cuadro final.
const TEXTO_FINAL: usize = 12;
const CUADRO_FINAL: usize = 15;
const GAME_OVER: usize = 16;
/// Text "SEGUNDOS".
const TEXTO_SEGUNDOS: usize = 17;
/// Zona roja: la pelota cayó (debajo de los pies).
const LIMITE: usize = 20;
/// Zona verde: donde la patada alcanza la pelota.
const LIMITE_CORRECTO: usize = 21;
const JUGADOR: usize = 22;
const BOMBA: usize = 24;

enum Inst {
    Sprite(Sprite),
    Texto(Texto),
}

pub struct Juego {
    pub proyecto: Proyecto,
    texturas: HashMap<String, Texture2D>,
    fuentes: Fuentes,
    audio: Audio,
    layout: usize,
    siguiente_layout: Option<String>,
    /// System.SetTimescale: el runtime arranca con 1.
    escala_tiempo: f64,
    /// Tiempo de juego (kahanTime): solo avanza con la escala de tiempo.
    tiempo_juego: f64,
    // Variables globales de "Game events" (no se reinician al cambiar de layout).
    fin: f64,
    contador_bomba: f64,
    velocidad_azar: f64,
    gravedad_pelota: f64,
    velocidad_subida: f64,
    tiempo_transcurrido: f64,
    /// Estado de la condición "Every 1 seconds" (Every_lastTime).
    cada_ultimo: f64,
    instancias: Vec<Inst>,
    /// Toques (y el ratón pulsado) en coordenadas del layout.
    toques: Vec<Vec2>,
}

impl Juego {
    pub fn nuevo(
        proyecto: Proyecto,
        texturas: HashMap<String, Texture2D>,
        fuentes: Fuentes,
        audio: Audio,
    ) -> Self {
        let primero = proyecto.primer_layout.clone();
        let mut juego = Self {
            proyecto,
            texturas,
            fuentes,
            audio,
            layout: 0,
            siguiente_layout: None,
            escala_tiempo: 1.0,
            tiempo_juego: 0.0,
            fin: 0.0,
            contador_bomba: 1.0,
            velocidad_azar: 0.0,
            gravedad_pelota: 400.0,
            velocidad_subida: 350.0,
            tiempo_transcurrido: 0.0,
            cada_ultimo: 0.0,
            instancias: Vec::new(),
            toques: Vec::new(),
        };
        juego.cambiar_layout(&primero);
        juego
    }

    // --- instancias --------------------------------------------------------------------------

    fn sprite(&self, i: usize) -> &Sprite {
        match &self.instancias[i] {
            Inst::Sprite(s) => s,
            Inst::Texto(_) => unreachable!("no es un Sprite"),
        }
    }

    fn sprite_mut(&mut self, i: usize) -> &mut Sprite {
        match &mut self.instancias[i] {
            Inst::Sprite(s) => s,
            Inst::Texto(_) => unreachable!("no es un Sprite"),
        }
    }

    fn texto_mut(&mut self, i: usize) -> &mut Texto {
        match &mut self.instancias[i] {
            Inst::Texto(t) => t,
            Inst::Sprite(_) => unreachable!("no es un Text"),
        }
    }

    /// Las instancias de un tipo, en orden de creación.
    fn de(&self, tipo: usize) -> Vec<usize> {
        (0..self.instancias.len())
            .filter(|&i| match &self.instancias[i] {
                Inst::Sprite(s) => s.tipo == tipo,
                Inst::Texto(t) => t.tipo == tipo,
            })
            .collect()
    }

    fn visible(&self, i: usize) -> bool {
        match &self.instancias[i] {
            Inst::Sprite(s) => s.visible,
            Inst::Texto(t) => t.visible,
        }
    }

    fn set_visible(&mut self, tipo: usize, visible: bool) {
        for i in self.de(tipo) {
            match &mut self.instancias[i] {
                Inst::Sprite(s) => s.visible = visible,
                Inst::Texto(t) => t.visible = visible,
            }
        }
    }

    fn anim_es(&self, i: usize, nombre: &str) -> bool {
        let s = self.sprite(i);
        s.anim_es(&self.proyecto.tipos[s.tipo], nombre)
    }

    fn set_anim(&mut self, i: usize, nombre: &str, desde: i32) {
        let tipo = self.sprite(i).tipo;
        let Self {
            proyecto,
            instancias,
            ..
        } = self;
        if let Inst::Sprite(s) = &mut instancias[i] {
            s.set_anim(&proyecto.tipos[tipo], nombre, desde);
        }
    }

    /// IsOverlapping: las instancias de `candidatas` que tocan alguna de `otro`.
    fn solapan(&self, candidatas: &[usize], otro: usize) -> Vec<usize> {
        let otros = self.de(otro);
        candidatas
            .iter()
            .copied()
            .filter(|&i| {
                let a = self.sprite(i);
                otros.iter().any(|&j| {
                    let b = self.sprite(j);
                    a.solapa(
                        &self.proyecto.tipos[a.tipo],
                        b,
                        &self.proyecto.tipos[b.tipo],
                    )
                })
            })
            .collect()
    }

    /// Condición invertida "not IsOverlapping": las que no tocan ninguna.
    fn no_solapan(&self, candidatas: &[usize], otro: usize) -> Vec<usize> {
        let si = self.solapan(candidatas, otro);
        candidatas
            .iter()
            .copied()
            .filter(|i| !si.contains(i))
            .collect()
    }

    /// Touch.IsTouchingObject: algún toque dentro de alguna instancia.
    fn tocando(&self, tipo: usize) -> Vec<usize> {
        self.de(tipo)
            .into_iter()
            .filter(|&i| {
                let s = self.sprite(i);
                self.toques
                    .iter()
                    .any(|&p| s.contiene(&self.proyecto.tipos[s.tipo], p))
            })
            .collect()
    }

    /// System.CreateObject: con las propiedades de la primera instancia del proyecto.
    fn crear(&mut self, tipo: usize, x: f32, y: f32) {
        let modelo = self.proyecto.instancia_por_defecto(tipo);
        // Capa 0 ("Juego") del layout actual.
        let mut s = Sprite::nuevo(modelo, &self.proyecto.tipos[tipo], 0);
        s.x = x;
        s.y = y;
        self.instancias.push(Inst::Sprite(s));
    }

    /// ClearDeathRow: quita las instancias destruidas.
    fn quitar_destruidas(&mut self) {
        self.instancias
            .retain(|i| !matches!(i, Inst::Sprite(s) if s.destruido));
    }

    /// Cambio de layout: se destruyen las instancias y se crean las del nuevo, y se dispara
    /// "On start of layout".
    fn cambiar_layout(&mut self, nombre: &str) {
        let indice = self
            .proyecto
            .layouts
            .iter()
            .position(|l| l.nombre.eq_ignore_ascii_case(nombre))
            .expect("layout");
        self.layout = indice;
        self.instancias.clear();
        for (c, capa) in self.proyecto.layouts[indice].capas.iter().enumerate() {
            for inst in &capa.instancias {
                let tipo = &self.proyecto.tipos[inst.tipo];
                self.instancias.push(if tipo.animaciones.is_empty() {
                    Inst::Texto(Texto::nuevo(inst, c))
                } else {
                    Inst::Sprite(Sprite::nuevo(inst, tipo, c))
                });
            }
        }
        self.al_empezar_layout();
    }

    fn en_juego(&self) -> bool {
        self.proyecto.layouts[self.layout]
            .nombre
            .eq_ignore_ascii_case("Dominar")
    }

    // --- entrada y tick ------------------------------------------------------------------------

    /// Toques entre ticks: actualiza la lista y lanza "On any touch start" por cada toque nuevo
    /// (el Touch del juego también toma el ratón).
    pub fn entrada(&mut self, activos: Vec<Vec2>, nuevos: usize) {
        self.toques = activos;
        for _ in 0..nuevos {
            self.al_tocar();
        }
    }

    /// Un tick del runtime con el dt del navegador (0 si pasó más de medio segundo, como máximo
    /// 0,1 s).
    pub fn tick(&mut self, dt_real: f64) {
        let dt = dt_real * self.escala_tiempo;
        self.tiempo_juego += dt;
        self.quitar_destruidas();
        // Behaviors: Platform se mueve antes de los eventos.
        for inst in &mut self.instancias {
            if let Inst::Sprite(s) = inst
                && let Some(p) = &mut s.plataforma
            {
                p.posttick(&mut s.y, dt);
            }
        }
        // Animaciones de los Sprites, con su disparador "On animation finished".
        for i in 0..self.instancias.len() {
            let tipo = match &self.instancias[i] {
                Inst::Sprite(s) => s.tipo,
                Inst::Texto(_) => continue,
            };
            let terminada = {
                let Self {
                    proyecto,
                    instancias,
                    ..
                } = self;
                match &mut instancias[i] {
                    Inst::Sprite(s) => s.tick(&proyecto.tipos[tipo], dt),
                    Inst::Texto(_) => None,
                }
            };
            if let Some(nombre) = terminada {
                self.sprite_mut(i).en_disparo = true;
                self.al_terminar_animacion(i, &nombre);
                let s = self.sprite_mut(i);
                s.en_disparo = false;
                s.despues_de_terminar();
            }
        }
        if let Some(nombre) = self.siguiente_layout.take() {
            self.cambiar_layout(&nombre);
        }
        // Lo destruido en los disparadores desaparece tras el primer evento de la hoja.
        self.quitar_destruidas();
        if self.en_juego() {
            self.eventos_juego();
        } else {
            self.eventos_start();
        }
    }

    /// System "Every X seconds" con el tiempo de juego.
    fn cada(&mut self, segundos: f64) -> bool {
        let ultimo = self.cada_ultimo;
        let ahora = self.tiempo_juego;
        if ahora >= ultimo + segundos {
            self.cada_ultimo = ultimo + segundos;
            if ahora >= self.cada_ultimo + 0.04 {
                self.cada_ultimo = ahora;
            }
            return true;
        }
        if ahora < ultimo - 0.1 {
            self.cada_ultimo = ahora;
        }
        false
    }

    // --- disparadores ------------------------------------------------------------------------

    fn al_empezar_layout(&mut self) {
        if !self.en_juego() {
            // Start events: On start of layout -> Audio.Preload (los sonidos ya están cargados).
            return;
        }
        // Game events, grupo "Start": On start of layout.
        for tipo in [
            REINICIAR,
            TEXTO_FINAL,
            CUADRO_FINAL,
            GAME_OVER,
            TEXTO_SEGUNDOS,
        ] {
            self.set_visible(tipo, false);
        }
        // t2.SetText(0): ese Text no tiene instancias.
        self.tiempo_transcurrido = 0.0;
        self.velocidad_azar = 0.0;
        for i in self.de(PELOTA) {
            set_gravedad(self.sprite_mut(i), 0.0);
        }
        self.fin = 0.0;
    }

    /// "On any touch start" (fuera del tick, como los eventos del DOM).
    fn al_tocar(&mut self) {
        if !self.en_juego() {
            return;
        }
        // Grupo "Start": el primer toque lanza la pelota y quita el texto.
        let empezar: Vec<usize> = self
            .de(TEXTO_EMPEZAR)
            .into_iter()
            .filter(|&i| self.visible(i))
            .collect();
        if !empezar.is_empty() {
            let (g, v) = (self.gravedad_pelota, self.velocidad_subida);
            for i in self.de(PELOTA) {
                let s = self.sprite_mut(i);
                set_gravedad(s, g);
                if let Some(p) = &mut s.plataforma {
                    p.dy = -v;
                }
            }
            for i in empezar {
                self.texto_mut(i).visible = false;
            }
            self.escala_tiempo = 1.0;
        }
        // Grupo "Personaje": patear (si no se está quemando).
        for i in self.de(JUGADOR) {
            if !self.anim_es(i, "quemando") {
                self.set_anim(i, "patear", 1);
            }
        }
    }

    /// "On animation finished" de un Sprite (durante su tick).
    fn al_terminar_animacion(&mut self, i: usize, nombre: &str) {
        let tipo = self.sprite(i).tipo;
        let es = |n: &str| nombre.eq_ignore_ascii_case(n);
        if tipo == BOMBA && es("explotar") {
            // Grupo "Bomba": destruirla (sigue existiendo hasta que se limpie).
            self.sprite_mut(i).destruido = true;
            // Grupo "Collisions": si explotó en la zona de los pies, se acabó.
            if !self.solapan(&[i], LIMITE).is_empty() {
                self.game_over();
                self.sprite_mut(i).destruido = true;
                self.fin_de_partida();
            }
        }
        if tipo == JUGADOR && es("patear") {
            // Grupo "Personaje" (diferido: estamos dentro del disparador).
            self.set_anim(i, "normal", 1);
        }
    }

    // --- eventos por tick ------------------------------------------------------------------------

    fn eventos_start(&mut self) {
        // Tocar "JUGAR".
        if !self.tocando(BOTON_JUGAR).is_empty() {
            self.siguiente_layout = Some("Dominar".to_owned());
            self.escala_tiempo = 0.0;
            self.audio.play("click", 1.0, "click");
            // Browser.GoToURL("javascript:inicio();"): esa función no existe en la página.
        }
    }

    fn eventos_juego(&mut self) {
        self.grupo_puntaje();
        self.grupo_pelota();
        self.grupo_bomba();
        self.grupo_collisions();
        self.grupo_game_over();
        self.grupo_personaje();
    }

    /// Grupo "Puntaje": un segundo más cada segundo de juego, y pelotas y bombas nuevas.
    fn grupo_puntaje(&mut self) {
        let cada = self.cada(1.0);
        let empezo = self.de(TEXTO_EMPEZAR).into_iter().any(|i| !self.visible(i));
        if !(cada && empezo) {
            return;
        }
        self.tiempo_transcurrido += 1.0;
        let t = self.tiempo_transcurrido;
        for i in self.de(TEXTO_TIEMPO) {
            self.texto_mut(i).set_texto_numero(t);
        }
        // Browser.GoToURL("javascript:tiempo(...)"): no existe en la página.
        let x_pelota = |j: &Self| j.de(PELOTA).first().map(|&i| j.sprite(i).x).unwrap_or(0.0);
        if t == 9.0 {
            let x = x_pelota(self);
            self.crear(PELOTA, x, 100.0);
            self.audio.play("aparicion", 1.0, "aparicion");
        }
        if t == 20.0 {
            self.velocidad_azar = 1.0;
        }
        if t == 30.0 {
            for i in self.de(PELOTA) {
                set_gravedad(self.sprite_mut(i), 450.0);
            }
            self.velocidad_azar = 2.0;
        }
        if t == 40.0 {
            let x = x_pelota(self);
            self.crear(PELOTA, x, 100.0);
            self.audio.play("aparicion", 1.0, "aparicion");
        }
        if t == 50.0 {
            let x = x_pelota(self);
            self.crear(BOMBA, x, 80.0);
            self.audio.play("aparicion", 1.0, "aparicion");
        }
        if self.contador_bomba == 0.0 {
            let x = x_pelota(self);
            self.crear(BOMBA, x, 80.0);
            self.audio.play("aparicion", 1.0, "aparicion");
        }
        if t >= 55.0 {
            self.contador_bomba -= 1.0;
            if self.contador_bomba < 0.0 {
                self.contador_bomba = 5.0;
            }
        }
    }

    /// Grupo "Pelota": caer en la zona roja y la patada.
    fn grupo_pelota(&mut self) {
        let pelotas = self.de(PELOTA);
        let caidas = self.solapan(&pelotas, LIMITE);
        if !caidas.is_empty() {
            for &i in &caidas {
                set_gravedad(self.sprite_mut(i), 0.0);
            }
            let sin_reiniciar = self.de(REINICIAR).into_iter().any(|i| !self.visible(i));
            if !self.audio.etiqueta_sonando("caida") && sin_reiniciar {
                self.audio.play("caida", 1.0, "caida");
            }
        }
        if self
            .de(JUGADOR)
            .into_iter()
            .any(|i| self.anim_es(i, "patear"))
        {
            let en_zona = self.solapan(&pelotas, LIMITE_CORRECTO);
            let pateadas = self.no_solapan(&en_zona, LIMITE);
            if !pateadas.is_empty() {
                if !self.audio.etiqueta_sonando("pegada") {
                    self.audio.play("pegada", 1.0, "pegada");
                }
                for &i in &pateadas {
                    // random(a, b) se evalúa por instancia.
                    let v = match self.velocidad_azar {
                        0.0 => Some(self.velocidad_subida),
                        1.0 => Some(rand::gen_range(250.0, 400.0)),
                        2.0 => Some(rand::gen_range(350.0, 450.0)),
                        _ => None,
                    };
                    if let (Some(v), Some(p)) = (v, &mut self.sprite_mut(i).plataforma) {
                        p.dy = -v;
                    }
                }
            }
        }
    }

    /// Grupo "Bomba": patearla la hace brillar; al caer brillando explota; si cae en la zona roja
    /// explota a los pies y quema al jugador.
    fn grupo_bomba(&mut self) {
        let bombas = self.de(BOMBA);
        if self
            .de(JUGADOR)
            .into_iter()
            .any(|i| self.anim_es(i, "patear"))
        {
            let en_zona = self.solapan(&bombas, LIMITE_CORRECTO);
            let pateadas = self.no_solapan(&en_zona, LIMITE);
            if !pateadas.is_empty() {
                let v = self.velocidad_subida;
                for &i in &pateadas {
                    if let Some(p) = &mut self.sprite_mut(i).plataforma {
                        p.dy = -v;
                    }
                    self.set_anim(i, "brillar", 1);
                }
                if !self.audio.etiqueta_sonando("pegada") {
                    self.audio.play("pegada", 1.0, "pegada");
                }
            }
        }
        let explotan: Vec<usize> = self
            .de(BOMBA)
            .into_iter()
            .filter(|&i| {
                self.sprite(i)
                    .plataforma
                    .as_ref()
                    .is_some_and(|p| p.cayendo())
                    && self.anim_es(i, "brillar")
            })
            .collect();
        if !explotan.is_empty() {
            for &i in &explotan {
                self.set_anim(i, "explotar", 1);
            }
            self.audio.play("explosion", 1.0, "explosion");
        }
        for i in self.de(BOMBA) {
            if self.anim_es(i, "explotar")
                && let Some(p) = &mut self.sprite_mut(i).plataforma
            {
                p.habilitada = false;
            }
        }
        let a_los_pies = self.solapan(&self.de(BOMBA), LIMITE);
        let jugadores: Vec<usize> = self
            .de(JUGADOR)
            .into_iter()
            .filter(|&i| !self.anim_es(i, "quemando"))
            .collect();
        if !a_los_pies.is_empty() && !jugadores.is_empty() {
            for &i in &a_los_pies {
                let tipo = self.sprite(i).tipo;
                let Self {
                    proyecto,
                    instancias,
                    ..
                } = self;
                if let Inst::Sprite(s) = &mut instancias[i] {
                    s.set_escala(&proyecto.tipos[tipo], 2.0);
                }
                self.set_anim(i, "explotar", 1);
            }
            for &i in &jugadores {
                self.set_anim(i, "quemando", 1);
            }
            self.audio.play("explosion", 1.0, "explosion");
        }
    }

    /// Grupo "Collisions": la pelota cae en la zona roja (mientras alguna bomba no esté
    /// explotando).
    fn grupo_collisions(&mut self) {
        let caidas = self.solapan(&self.de(PELOTA), LIMITE);
        let bomba_quieta = self
            .de(BOMBA)
            .into_iter()
            .any(|i| !self.anim_es(i, "explotar"));
        if !caidas.is_empty() && bomba_quieta {
            self.game_over();
            self.fin_de_partida();
        }
    }

    fn game_over(&mut self) {
        for tipo in [CUADRO_FINAL, GAME_OVER, REINICIAR, TEXTO_FINAL] {
            self.set_visible(tipo, true);
        }
        let tiempo = self
            .de(TEXTO_TIEMPO)
            .first()
            .map(|&i| match &self.instancias[i] {
                Inst::Texto(t) => t.texto.clone(),
                Inst::Sprite(_) => String::new(),
            })
            .unwrap_or_default();
        for i in self.de(TEXTO_FINAL) {
            self.texto_mut(i).texto = tiempo.clone();
        }
        self.set_visible(TEXTO_SEGUNDOS, true);
        self.escala_tiempo = 0.0;
    }

    /// Subevento: la primera vez, Browser.GoToURL("javascript:fin(...)") (no existe en la
    /// página) y fin = 1.
    fn fin_de_partida(&mut self) {
        if self.fin == 0.0 {
            self.fin = 1.0;
        }
    }

    /// Grupo "Game Over".
    fn grupo_game_over(&mut self) {
        let ok = self.tocando(REINICIAR);
        if ok.iter().any(|&i| self.visible(i)) {
            self.audio.play("click", 1.0, "click");
            // Browser.GoToURL("javascript:reiniciar();"): no existe en la página.
            self.siguiente_layout = Some("Start".to_owned());
        }
        // Un evento sin condiciones: suena "click" en cada tick de la partida (y se llama a
        // "javascript:compartir(...)", que tampoco existe). Seguramente quedó así al borrar un
        // botón de compartir.
        self.audio.play("click", 1.0, "click");
    }

    /// Grupo "Personaje": el jugador mira la pelota subir o bajar.
    fn grupo_personaje(&mut self) {
        let pelotas = self.de(PELOTA);
        let subiendo = pelotas.iter().any(|&i| {
            self.sprite(i)
                .plataforma
                .as_ref()
                .is_some_and(|p| p.subiendo())
        });
        if subiendo {
            for i in self.de(JUGADOR) {
                if !self.anim_es(i, "mirar")
                    && !self.anim_es(i, "patear")
                    && !self.anim_es(i, "quemando")
                {
                    self.set_anim(i, "mirar", 1);
                }
            }
        }
        let cayendo = pelotas.iter().any(|&i| {
            self.sprite(i)
                .plataforma
                .as_ref()
                .is_some_and(|p| p.cayendo())
        });
        if cayendo {
            for i in self.de(JUGADOR) {
                if !self.anim_es(i, "bajar")
                    && !self.anim_es(i, "patear")
                    && !self.anim_es(i, "quemando")
                {
                    self.set_anim(i, "bajar", 1);
                }
            }
        }
    }

    // --- dibujo ---------------------------------------------------------------------------------

    /// Layout.draw: cada capa (las opacas se pintan de su color) con sus instancias en orden.
    pub fn dibujar(&self) {
        let layout = &self.proyecto.layouts[self.layout];
        for (c, capa) in layout.capas.iter().enumerate() {
            if !capa.visible {
                continue;
            }
            if !capa.transparente {
                clear_background(capa.fondo);
            }
            for inst in &self.instancias {
                match inst {
                    Inst::Sprite(s) if s.capa == c => {
                        s.dibujar(&self.proyecto.tipos[s.tipo], &self.texturas)
                    }
                    Inst::Texto(t) if t.capa == c => t.dibujar(&self.fuentes),
                    _ => {}
                }
            }
        }
    }
}

/// Platform.SetGravity.
fn set_gravedad(s: &mut Sprite, g: f64) {
    if let Some(p) = &mut s.plataforma {
        p.gravedad = g;
    }
}
