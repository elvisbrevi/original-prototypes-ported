// El tick del runtime de Construct 2 y la hoja de eventos del juego ("InGame"), traducidos a mano
// desde data.js (tools/c2_eventos.py; volcado en reference/eventos.txt). La exportación
// minificada perdió los nombres de los tipos de objeto (t0, t1...): los de aquí son los de
// reference/nombres.json.

use construct2::geometria::redondear;
use construct2::texto::numero_a_texto;
use construct2::{Audio, Cada, Entrada, MemoriaColision, Mundo};
use macroquad::math::DVec2;
use macroquad::prelude::rand;

// Tipos de objeto (índice en data.js).
const SUELO: usize = 0;
const RUEDA_DELANTERA: usize = 1;
const MOTO: usize = 4;
const RUEDA_TRASERA: usize = 5;
const CABEZA: usize = 7;
const SPAWN: usize = 8;
const CERROS: [usize; 4] = [9, 10, 11, 12];
const TEXTO_TIEMPO: usize = 13;
const META: usize = 14;
const MAPA_META: usize = 15;
const MOTO_MAPA: usize = 16;
const TEXTO_COMENZAR: usize = 17;
const BOTON_COMPARTIR: usize = 18;
const BOTON_REINTENTAR: usize = 19;
const TEXTO_RESULTADO: usize = 20;
const POLVO: usize = 24;

/// Capas del layout "InGame".
const CAPA_PLAYER: usize = 1;
const CAPA_GUI: usize = 2;
const CAPA_GUI_FINAL: usize = 3;

const DISTANCIA_SPAWN: f64 = 600.0;
const MAX_VELOCIDAD_MOTO: f64 = 2800.0;

/// Las teclas que mira el juego (Keyboard.IsKeyDown).
#[derive(Clone, Copy, Default)]
pub struct Teclas {
    pub arriba: bool,
    pub izquierda: bool,
    pub derecha: bool,
    pub w: bool,
    pub a: bool,
    pub d: bool,
}

/// Las variables globales: ningún evento las reinicia (RestartLayout no las toca).
#[derive(Default)]
struct Globales {
    acelerando: f64,
    velocidad_moto: f64,
    termino: f64,
    tiempo: f64,
    empezo: f64,
}

pub struct Juego {
    mundo: Mundo,
    audio: Audio,
    g: Globales,
    /// System.RestartLayout pendiente (se cambia en el tick siguiente).
    reiniciar: bool,
    /// System.IsOnMobile.
    movil: bool,
    /// Touch.Beta (inclinación del dispositivo, en grados).
    beta: f64,
    teclas: Teclas,
    /// Toques activos (con el ratón pulsado: el Touch del juego usa el ratón).
    toques: Vec<DVec2>,
    /// El primer tick de un layout (TriggerOnce y el arranque).
    primer_tick: bool,
    // Estado de las condiciones.
    cada_segundo: Cada,
    ultimo_trigger_once: i64,
    choques_cabeza: [MemoriaColision; 5],
    choques_rueda: [MemoriaColision; 5],
}

impl Juego {
    pub fn nuevo(mundo: Mundo, audio: Audio, movil: bool) -> Self {
        let primero = mundo.proyecto.primer_layout.clone();
        let mut juego = Self {
            mundo,
            audio,
            g: Globales::default(),
            reiniciar: false,
            movil,
            beta: 0.0,
            teclas: Teclas::default(),
            toques: Vec::new(),
            primer_tick: true,
            cada_segundo: Cada::default(),
            ultimo_trigger_once: -1,
            choques_cabeza: Default::default(),
            choques_rueda: Default::default(),
        };
        juego.cambiar_layout(&primero);
        juego
    }

    // --- acciones comunes ----------------------------------------------------------------------

    fn primera_posicion(&self, tipo: usize) -> (f64, f64) {
        self.mundo
            .primera(tipo)
            .map(|i| self.mundo.posicion(i))
            .unwrap_or((0.0, 0.0))
    }

    fn set_x_tipo(&mut self, tipo: usize, x: f64) {
        for i in self.mundo.de(tipo) {
            let y = self.mundo.posicion(i).1;
            self.mundo.set_posicion(i, x, y);
        }
    }

    fn velocidad_angular_tipo(&mut self, tipo: usize, grados: f64) {
        for i in self.mundo.de(tipo) {
            self.mundo.velocidad_angular(i, grados);
        }
    }

    fn pulverizar(&mut self, si: bool) {
        for i in self.mundo.de(POLVO) {
            self.mundo.particulas_mut(i).pulverizando = si;
        }
    }

    fn textos(&mut self, tipo: usize, texto: String) {
        for i in self.mundo.de(tipo) {
            self.mundo.texto_mut(i).texto = texto.clone();
        }
    }

    /// System.RestartLayout: solo cuenta si no hay ya un cambio pendiente.
    fn reiniciar_layout(&mut self) {
        self.reiniciar = true;
    }

    /// Touch.IsTouchingObject: algún toque sobre una instancia (en las coordenadas de su capa,
    /// aunque la capa esté oculta).
    fn tocando(&self, tipo: usize) -> bool {
        self.toques
            .iter()
            .any(|&p| !self.mundo.en_punto(tipo, p).is_empty())
    }

    fn acelerar(&self) -> bool {
        !self.toques.is_empty() || self.teclas.arriba || self.teclas.w
    }

    // --- entrada y tick ------------------------------------------------------------------------

    /// Toques, ratón y teclado entre ticks.
    pub fn entrada(&mut self, e: &Entrada, teclas: Teclas, beta: f64) {
        self.toques = e.activos.clone();
        self.teclas = teclas;
        self.beta = beta;
        // Mouse "On object clicked" (izquierdo, simple) sobre Reintentar: es un disparador del
        // bloque O del grupo "GUI Final", aunque la capa esté oculta.
        if let Some(p) = e.clic
            && !self.mundo.en_punto(BOTON_REINTENTAR, p).is_empty()
        {
            self.reiniciar_layout();
        }
    }

    /// Un tick del runtime con el dt del navegador (construct2::dt_del_runtime).
    pub fn tick(&mut self, dt_real: f64) {
        self.mundo.avanzar(dt_real);
        self.mundo.quitar_destruidas();
        // Comportamientos: Physics avanza el mundo de Box2D en el primero; Wrap mueve el suelo.
        self.mundo.tick_comportamientos();
        // Objetos con tick: las partículas.
        for i in 0..self.mundo.instancias.len() {
            self.mundo.tick_animacion(i);
        }
        if std::mem::take(&mut self.reiniciar) {
            let layout = self.mundo.nombre_layout().to_owned();
            self.cambiar_layout(&layout);
        }
        self.mundo.quitar_destruidas();
        self.grupo_gui();
        self.mundo.quitar_destruidas();
        self.grupo_gameplay();
        self.mundo.quitar_destruidas();
        self.primer_tick = false;
        self.mundo.olvidar_colisiones();
        // tick2: ScrollTo lleva la vista a la moto.
        self.mundo.tick2();
        self.mundo.ticks += 1;
    }

    fn cambiar_layout(&mut self, nombre: &str) {
        self.mundo.cambiar_layout(nombre);
        self.primer_tick = true;
        self.al_empezar_layout();
    }

    /// Grupo "Inicio", On start of layout.
    fn al_empezar_layout(&mut self) {
        // Audio.Preload: los sonidos ya están cargados.
        let texto = format!("Tiempo: {} segundos", numero_a_texto(self.g.tiempo));
        self.textos(TEXTO_TIEMPO, texto);
        for tipo in [RUEDA_DELANTERA, RUEDA_TRASERA] {
            if !self.mundo.de(tipo).is_empty() {
                self.mundo.gravedad_mundo(10.0);
            }
        }
        for moto in self.mundo.de(MOTO) {
            for (punto, tipo, limites) in [
                (1, RUEDA_DELANTERA, None),
                (2, RUEDA_TRASERA, None),
                (3, CABEZA, Some((0.0, 0.0))),
            ] {
                if let Some(objetivo) = self.mundo.primera(tipo) {
                    self.mundo
                        .crear_articulacion(moto, punto, objetivo, limites);
                }
            }
        }
        self.velocidad_angular_tipo(RUEDA_DELANTERA, 0.0);
        self.velocidad_angular_tipo(RUEDA_TRASERA, 0.0);
        self.set_x_tipo(META, 11000.0);
        self.mundo.set_capa_visible(CAPA_GUI_FINAL, false);
        self.mundo.set_capa_visible(CAPA_GUI, true);
        self.g.termino = 0.0;
        self.g.velocidad_moto = 0.0;
        self.pulverizar(false);
        if self.g.empezo == 1.0 {
            self.mundo.destruir_tipo(TEXTO_COMENZAR);
        }
    }

    // --- grupo "GUI" ---------------------------------------------------------------------------

    fn grupo_gui(&mut self) {
        // "GUI Final": el bloque O tiene también Touch.IsTouchingObject(Compartir).
        if self.tocando(BOTON_COMPARTIR) {
            self.reiniciar_layout();
        }
        // "Mapa": la marca de la moto en la barra de arriba.
        let (meta_x, _) = self.primera_posicion(MAPA_META);
        let (moto_x, _) = self.primera_posicion(MOTO);
        self.set_x_tipo(MOTO_MAPA, meta_x + moto_x / 100.0);
        // "GUI Inicio": arriba, W o un toque empiezan la carrera.
        if self.acelerar() && self.g.termino == 0.0 {
            self.g.empezo = 1.0;
            self.mundo.destruir_tipo(TEXTO_COMENZAR);
        }
        if self.g.empezo == 1.0 && self.cada_segundo.cumple(1.0, self.mundo.tiempo()) {
            self.g.tiempo += 1.0;
            let texto = format!("Tiempo: {} segundos", numero_a_texto(self.g.tiempo));
            self.textos(TEXTO_TIEMPO, texto);
        }
    }

    // --- grupo "Gameplay" ----------------------------------------------------------------------

    /// System.TriggerOnce: cierto en el primer tick seguido en que se evalúa.
    fn trigger_once(&mut self) -> bool {
        let tick = self.mundo.ticks as i64;
        let ultimo = std::mem::replace(&mut self.ultimo_trigger_once, tick);
        self.primer_tick || ultimo != tick - 1
    }

    fn grupo_gameplay(&mut self) {
        if self.g.empezo != 1.0 {
            return;
        }
        // El polvo sale de la rueda trasera.
        if let Some(r) = self.mundo.primera(RUEDA_TRASERA) {
            let (x, y) = self.mundo.posicion(r);
            for i in self.mundo.de(POLVO) {
                self.mundo.set_posicion(i, x, y);
            }
        }
        self.grupo_moto();
        self.mundo.quitar_destruidas();
        self.grupo_spawn();
        self.mundo.quitar_destruidas();
        self.grupo_terminar();
    }

    /// Grupo "Moto": girar, acelerar, la cabeza contra el suelo y el polvo.
    fn grupo_moto(&mut self) {
        let t = self.teclas;
        if t.derecha || t.d {
            self.velocidad_angular_tipo(MOTO, 200.0);
        }
        if t.izquierda || t.a {
            self.velocidad_angular_tipo(MOTO, -200.0);
        }
        if self.movil {
            let v = self.beta * 5.0;
            self.velocidad_angular_tipo(MOTO, v);
        }
        if self.acelerar() {
            let v = self.g.velocidad_moto;
            self.velocidad_angular_tipo(RUEDA_DELANTERA, v);
            self.velocidad_angular_tipo(RUEDA_TRASERA, v);
            if self.audio.en_silencio() && self.g.acelerando == 0.0 {
                self.audio.detener("acelerar");
                self.audio.play("acelerar", false, 1.0, "acelerar");
                self.g.acelerando = 1.0;
                if !self.audio.etiqueta_sonando("motor") {
                    self.audio.play("motor2", true, 1.0, "motor");
                }
            }
            if self.g.velocidad_moto < MAX_VELOCIDAD_MOTO {
                self.g.velocidad_moto += 15.0;
            }
        }
        if !self.acelerar() {
            self.g.acelerando = 0.0;
            self.pulverizar(false);
            if self.g.velocidad_moto > 10.0 {
                self.g.velocidad_moto -= 10.0;
            }
        }
        if !t.izquierda && !t.derecha && self.trigger_once() {
            self.velocidad_angular_tipo(RUEDA_DELANTERA, 0.0);
            self.velocidad_angular_tipo(RUEDA_TRASERA, 0.0);
        }
        // Bloque O: la cabeza choca con el suelo o un cerro.
        let cabezas = self.mundo.de(CABEZA);
        let mut golpe = false;
        for (k, tipo) in [SUELO, CERROS[0], CERROS[1], CERROS[2], CERROS[3]]
            .into_iter()
            .enumerate()
        {
            if !self.choques_cabeza[k]
                .nuevas(&self.mundo, &cabezas, tipo)
                .is_empty()
            {
                golpe = true;
            }
        }
        if golpe {
            self.reiniciar_layout();
        }
        // Bloque O: la rueda trasera toca un cerro o el suelo.
        let ruedas = self.mundo.de(RUEDA_TRASERA);
        let mut toca = false;
        for (k, tipo) in [CERROS[0], CERROS[1], CERROS[2], CERROS[3], SUELO]
            .into_iter()
            .enumerate()
        {
            if !self.choques_rueda[k]
                .nuevas(&self.mundo, &ruedas, tipo)
                .is_empty()
            {
                toca = true;
            }
        }
        if toca {
            self.pulverizar(true);
        }
    }

    /// Grupo "Spawn": cuando la moto pasa 600 px de un spawn, cada subevento tira su propio
    /// round(random(0, 4)) y puede crear un cerro 600 px por delante.
    fn grupo_spawn(&mut self) {
        let (moto_x, _) = self.primera_posicion(MOTO);
        let elegidos: Vec<usize> = self
            .mundo
            .de(SPAWN)
            .into_iter()
            .filter(|&i| self.mundo.posicion(i).0 < moto_x - DISTANCIA_SPAWN)
            .collect();
        if elegidos.is_empty() {
            return;
        }
        for (k, &cerro) in CERROS.iter().enumerate() {
            let tirada = redondear(rand::gen_range(0.0, 1.0) * 4.0);
            if tirada != k as f64 {
                continue;
            }
            let (_, y) = self.primera_posicion(cerro);
            self.mundo
                .crear(cerro, CAPA_PLAYER, moto_x + DISTANCIA_SPAWN, y);
            for &i in &elegidos {
                let sy = self.mundo.posicion(i).1;
                self.mundo.set_posicion(i, moto_x + DISTANCIA_SPAWN, sy);
            }
        }
    }

    /// Grupo "Terminar": la meta está en x = 11000.
    fn grupo_terminar(&mut self) {
        let (moto_x, _) = self.primera_posicion(MOTO);
        if moto_x >= 10000.0 {
            self.mundo.destruir_tipo(SPAWN);
        }
        let (meta_x, _) = self.primera_posicion(META);
        if moto_x >= meta_x {
            self.g.termino = 1.0;
            self.g.empezo = 0.0;
            self.mundo.set_capa_visible(CAPA_GUI_FINAL, true);
            self.mundo.set_capa_visible(CAPA_GUI, false);
            let texto = format!("Tu tiempo fue: {} segundos.", numero_a_texto(self.g.tiempo));
            self.textos(TEXTO_RESULTADO, texto);
            self.g.tiempo = 0.0;
        }
    }

    // --- dibujo --------------------------------------------------------------------------------

    pub fn dibujar(&self) {
        self.mundo.dibujar();
    }
}
