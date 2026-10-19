// El tick del runtime de Construct 2 y las dos hojas de eventos del juego ("Start events" y
// "Game events"), traducidas a mano desde data.js (tools/c2_eventos.py; volcado en
// reference/eventos.txt). Los tipos de objeto se llaman t0, t1... porque la exportación
// minificada perdió los nombres; aquí se nombran por su imagen.

use construct2::{Audio, Cada, Entrada, Mundo};
use macroquad::math::DVec2;
use macroquad::prelude::rand;

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

pub struct Juego {
    mundo: Mundo,
    audio: Audio,
    siguiente_layout: Option<String>,
    // Variables globales de "Game events" (no se reinician al cambiar de layout).
    fin: f64,
    contador_bomba: f64,
    velocidad_azar: f64,
    gravedad_pelota: f64,
    velocidad_subida: f64,
    tiempo_transcurrido: f64,
    /// Estado de la condición "Every 1 seconds".
    cada: Cada,
    /// Toques (y el ratón pulsado) en coordenadas del layout.
    toques: Vec<DVec2>,
}

impl Juego {
    pub fn nuevo(mundo: Mundo, audio: Audio) -> Self {
        let primero = mundo.proyecto.primer_layout.clone();
        let mut juego = Self {
            mundo,
            audio,
            siguiente_layout: None,
            fin: 0.0,
            contador_bomba: 1.0,
            velocidad_azar: 0.0,
            gravedad_pelota: 400.0,
            velocidad_subida: 350.0,
            tiempo_transcurrido: 0.0,
            cada: Cada::default(),
            toques: Vec::new(),
        };
        juego.cambiar_layout(&primero);
        juego
    }

    // --- instancias --------------------------------------------------------------------------

    /// Touch.IsTouchingObject: algún toque dentro de alguna instancia.
    fn tocando(&self, tipo: usize) -> Vec<usize> {
        self.mundo
            .de(tipo)
            .into_iter()
            .filter(|&i| self.toques.iter().any(|&p| self.mundo.contiene(i, p)))
            .collect()
    }

    /// Platform.SetGravity.
    fn set_gravedad(&mut self, i: usize, g: f64) {
        if let Some(p) = self.mundo.sprite_mut(i).plataforma_mut() {
            p.gravedad = g;
        }
    }

    /// Platform.SetVectorY.
    fn set_vector_y(&mut self, i: usize, dy: f64) {
        if let Some(p) = self.mundo.sprite_mut(i).plataforma_mut() {
            p.dy = dy;
        }
    }

    /// Cambio de layout: se destruyen las instancias y se crean las del nuevo, y se dispara
    /// "On start of layout".
    fn cambiar_layout(&mut self, nombre: &str) {
        self.mundo.cambiar_layout(nombre);
        self.al_empezar_layout();
    }

    fn en_juego(&self) -> bool {
        self.mundo.en_layout("Dominar")
    }

    // --- entrada y tick ------------------------------------------------------------------------

    /// Toques entre ticks: actualiza la lista y lanza "On any touch start" por cada toque nuevo
    /// (el Touch del juego también toma el ratón).
    pub fn entrada(&mut self, e: &Entrada) {
        self.toques = e.activos.clone();
        for _ in &e.inicios {
            self.al_tocar();
        }
    }

    /// Un tick del runtime con el dt del navegador (construct2::dt_del_runtime).
    pub fn tick(&mut self, dt_real: f64) {
        self.mundo.avanzar(dt_real);
        self.mundo.quitar_destruidas();
        // Behaviors: Platform se mueve antes de los eventos.
        self.mundo.tick_comportamientos();
        // Animaciones de los Sprites, con su disparador "On animation finished".
        for i in 0..self.mundo.instancias.len() {
            if let Some(nombre) = self.mundo.tick_animacion(i) {
                self.mundo.sprite_mut(i).en_disparo = true;
                self.al_terminar_animacion(i, &nombre);
                let s = self.mundo.sprite_mut(i);
                s.en_disparo = false;
                s.despues_de_terminar();
            }
        }
        if let Some(nombre) = self.siguiente_layout.take() {
            self.cambiar_layout(&nombre);
        }
        // Lo destruido en los disparadores desaparece tras el primer evento de la hoja.
        self.mundo.quitar_destruidas();
        if self.en_juego() {
            self.eventos_juego();
        } else {
            self.eventos_start();
        }
        self.mundo.ticks += 1;
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
            self.mundo.set_visible_tipo(tipo, false);
        }
        // t2.SetText(0): ese Text no tiene instancias.
        self.tiempo_transcurrido = 0.0;
        self.velocidad_azar = 0.0;
        for i in self.mundo.de(PELOTA) {
            self.set_gravedad(i, 0.0);
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
            .mundo
            .de(TEXTO_EMPEZAR)
            .into_iter()
            .filter(|&i| self.mundo.visible(i))
            .collect();
        if !empezar.is_empty() {
            let (g, v) = (self.gravedad_pelota, self.velocidad_subida);
            for i in self.mundo.de(PELOTA) {
                self.set_gravedad(i, g);
                self.set_vector_y(i, -v);
            }
            for i in empezar {
                self.mundo.set_visible(i, false);
            }
            self.mundo.escala_tiempo = 1.0;
        }
        // Grupo "Personaje": patear (si no se está quemando).
        for i in self.mundo.de(JUGADOR) {
            if !self.mundo.anim_es(i, "quemando") {
                self.mundo.set_anim(i, "patear", 1);
            }
        }
    }

    /// "On animation finished" de un Sprite (durante su tick).
    fn al_terminar_animacion(&mut self, i: usize, nombre: &str) {
        let tipo = self.mundo.sprite(i).tipo;
        let es = |n: &str| nombre.eq_ignore_ascii_case(n);
        if tipo == BOMBA && es("explotar") {
            // Grupo "Bomba": destruirla (sigue existiendo hasta que se limpie).
            self.mundo.destruir(i);
            // Grupo "Collisions": si explotó en la zona de los pies, se acabó.
            if !self.mundo.solapan(&[i], LIMITE).is_empty() {
                self.game_over();
                self.mundo.destruir(i);
                self.fin_de_partida();
            }
        }
        if tipo == JUGADOR && es("patear") {
            // Grupo "Personaje" (diferido: estamos dentro del disparador).
            self.mundo.set_anim(i, "normal", 1);
        }
    }

    // --- eventos por tick ------------------------------------------------------------------------

    fn eventos_start(&mut self) {
        // Tocar "JUGAR".
        if !self.tocando(BOTON_JUGAR).is_empty() {
            self.siguiente_layout = Some("Dominar".to_owned());
            self.mundo.escala_tiempo = 0.0;
            self.audio.play("click", false, 1.0, "click");
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
        let cada = self.cada.cumple(1.0, self.mundo.tiempo());
        let empezo = self
            .mundo
            .de(TEXTO_EMPEZAR)
            .into_iter()
            .any(|i| !self.mundo.visible(i));
        if !(cada && empezo) {
            return;
        }
        self.tiempo_transcurrido += 1.0;
        let t = self.tiempo_transcurrido;
        for i in self.mundo.de(TEXTO_TIEMPO) {
            self.mundo.texto_mut(i).set_texto_numero(t);
        }
        // Browser.GoToURL("javascript:tiempo(...)"): no existe en la página.
        let x_pelota = |j: &Self| {
            j.mundo
                .primera(PELOTA)
                .map(|i| j.mundo.sprite(i).x)
                .unwrap_or(0.0)
        };
        if t == 9.0 {
            let x = x_pelota(self);
            self.mundo.crear(PELOTA, 0, x, 100.0);
            self.audio.play("aparicion", false, 1.0, "aparicion");
        }
        if t == 20.0 {
            self.velocidad_azar = 1.0;
        }
        if t == 30.0 {
            for i in self.mundo.de(PELOTA) {
                self.set_gravedad(i, 450.0);
            }
            self.velocidad_azar = 2.0;
        }
        if t == 40.0 {
            let x = x_pelota(self);
            self.mundo.crear(PELOTA, 0, x, 100.0);
            self.audio.play("aparicion", false, 1.0, "aparicion");
        }
        if t == 50.0 {
            let x = x_pelota(self);
            self.mundo.crear(BOMBA, 0, x, 80.0);
            self.audio.play("aparicion", false, 1.0, "aparicion");
        }
        if self.contador_bomba == 0.0 {
            let x = x_pelota(self);
            self.mundo.crear(BOMBA, 0, x, 80.0);
            self.audio.play("aparicion", false, 1.0, "aparicion");
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
        let pelotas = self.mundo.de(PELOTA);
        let caidas = self.mundo.solapan(&pelotas, LIMITE);
        if !caidas.is_empty() {
            for &i in &caidas {
                self.set_gravedad(i, 0.0);
            }
            let sin_reiniciar = self
                .mundo
                .de(REINICIAR)
                .into_iter()
                .any(|i| !self.mundo.visible(i));
            if !self.audio.etiqueta_sonando("caida") && sin_reiniciar {
                self.audio.play("caida", false, 1.0, "caida");
            }
        }
        if self
            .mundo
            .de(JUGADOR)
            .into_iter()
            .any(|i| self.mundo.anim_es(i, "patear"))
        {
            let en_zona = self.mundo.solapan(&pelotas, LIMITE_CORRECTO);
            let pateadas = self.mundo.no_solapan(&en_zona, LIMITE);
            if !pateadas.is_empty() {
                if !self.audio.etiqueta_sonando("pegada") {
                    self.audio.play("pegada", false, 1.0, "pegada");
                }
                for &i in &pateadas {
                    // random(a, b) se evalúa por instancia.
                    let v = match self.velocidad_azar {
                        0.0 => Some(self.velocidad_subida),
                        1.0 => Some(rand::gen_range(250.0, 400.0)),
                        2.0 => Some(rand::gen_range(350.0, 450.0)),
                        _ => None,
                    };
                    if let Some(v) = v {
                        self.set_vector_y(i, -v);
                    }
                }
            }
        }
    }

    /// Grupo "Bomba": patearla la hace brillar; al caer brillando explota; si cae en la zona roja
    /// explota a los pies y quema al jugador.
    fn grupo_bomba(&mut self) {
        let bombas = self.mundo.de(BOMBA);
        if self
            .mundo
            .de(JUGADOR)
            .into_iter()
            .any(|i| self.mundo.anim_es(i, "patear"))
        {
            let en_zona = self.mundo.solapan(&bombas, LIMITE_CORRECTO);
            let pateadas = self.mundo.no_solapan(&en_zona, LIMITE);
            if !pateadas.is_empty() {
                let v = self.velocidad_subida;
                for &i in &pateadas {
                    self.set_vector_y(i, -v);
                    self.mundo.set_anim(i, "brillar", 1);
                }
                if !self.audio.etiqueta_sonando("pegada") {
                    self.audio.play("pegada", false, 1.0, "pegada");
                }
            }
        }
        let explotan: Vec<usize> = self
            .mundo
            .de(BOMBA)
            .into_iter()
            .filter(|&i| {
                self.mundo
                    .sprite(i)
                    .plataforma()
                    .is_some_and(|p| p.cayendo())
                    && self.mundo.anim_es(i, "brillar")
            })
            .collect();
        if !explotan.is_empty() {
            for &i in &explotan {
                self.mundo.set_anim(i, "explotar", 1);
            }
            self.audio.play("explosion", false, 1.0, "explosion");
        }
        for i in self.mundo.de(BOMBA) {
            if self.mundo.anim_es(i, "explotar")
                && let Some(p) = self.mundo.sprite_mut(i).plataforma_mut()
            {
                p.habilitada = false;
            }
        }
        let a_los_pies = self.mundo.solapan(&self.mundo.de(BOMBA), LIMITE);
        let jugadores: Vec<usize> = self
            .mundo
            .de(JUGADOR)
            .into_iter()
            .filter(|&i| !self.mundo.anim_es(i, "quemando"))
            .collect();
        if !a_los_pies.is_empty() && !jugadores.is_empty() {
            for &i in &a_los_pies {
                self.mundo.set_escala(i, 2.0);
                self.mundo.set_anim(i, "explotar", 1);
            }
            for &i in &jugadores {
                self.mundo.set_anim(i, "quemando", 1);
            }
            self.audio.play("explosion", false, 1.0, "explosion");
        }
    }

    /// Grupo "Collisions": la pelota cae en la zona roja (mientras alguna bomba no esté
    /// explotando).
    fn grupo_collisions(&mut self) {
        let caidas = self.mundo.solapan(&self.mundo.de(PELOTA), LIMITE);
        let bomba_quieta = self
            .mundo
            .de(BOMBA)
            .into_iter()
            .any(|i| !self.mundo.anim_es(i, "explotar"));
        if !caidas.is_empty() && bomba_quieta {
            self.game_over();
            self.fin_de_partida();
        }
    }

    fn game_over(&mut self) {
        for tipo in [CUADRO_FINAL, GAME_OVER, REINICIAR, TEXTO_FINAL] {
            self.mundo.set_visible_tipo(tipo, true);
        }
        let tiempo = self
            .mundo
            .primera(TEXTO_TIEMPO)
            .map(|i| self.mundo.texto(i).texto.clone())
            .unwrap_or_default();
        for i in self.mundo.de(TEXTO_FINAL) {
            self.mundo.texto_mut(i).texto = tiempo.clone();
        }
        self.mundo.set_visible_tipo(TEXTO_SEGUNDOS, true);
        self.mundo.escala_tiempo = 0.0;
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
        if ok.iter().any(|&i| self.mundo.visible(i)) {
            self.audio.play("click", false, 1.0, "click");
            // Browser.GoToURL("javascript:reiniciar();"): no existe en la página.
            self.siguiente_layout = Some("Start".to_owned());
        }
        // Un evento sin condiciones: suena "click" en cada tick de la partida (y se llama a
        // "javascript:compartir(...)", que tampoco existe). Seguramente quedó así al borrar un
        // botón de compartir.
        self.audio.play("click", false, 1.0, "click");
    }

    /// Grupo "Personaje": el jugador mira la pelota subir o bajar.
    fn grupo_personaje(&mut self) {
        let pelotas = self.mundo.de(PELOTA);
        let subiendo = pelotas.iter().any(|&i| {
            self.mundo
                .sprite(i)
                .plataforma()
                .is_some_and(|p| p.subiendo())
        });
        if subiendo {
            for i in self.mundo.de(JUGADOR) {
                if !self.mundo.anim_es(i, "mirar")
                    && !self.mundo.anim_es(i, "patear")
                    && !self.mundo.anim_es(i, "quemando")
                {
                    self.mundo.set_anim(i, "mirar", 1);
                }
            }
        }
        let cayendo = pelotas.iter().any(|&i| {
            self.mundo
                .sprite(i)
                .plataforma()
                .is_some_and(|p| p.cayendo())
        });
        if cayendo {
            for i in self.mundo.de(JUGADOR) {
                if !self.mundo.anim_es(i, "bajar")
                    && !self.mundo.anim_es(i, "patear")
                    && !self.mundo.anim_es(i, "quemando")
                {
                    self.mundo.set_anim(i, "bajar", 1);
                }
            }
        }
    }

    // --- dibujo ---------------------------------------------------------------------------------

    pub fn dibujar(&self) {
        self.mundo.dibujar();
    }
}
