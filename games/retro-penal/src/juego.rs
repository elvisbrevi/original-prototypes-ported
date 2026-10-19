// El tick del runtime de Construct 2 y las tres hojas de eventos del juego ("Menu Principal",
// "Juego" y "Game Over"), traducidas a mano desde data.js (tools/c2_eventos.py; volcado en
// reference/eventos.txt). La exportación minificada perdió los nombres de los tipos de objeto
// (t0, t1...): los de aquí son los de reference/nombres.json.

use construct2::geometria::grados_de;
use construct2::texto::numero_a_texto;
use construct2::{Audio, Cada, Cursor, Entrada, Esperas, MemoriaColision, Mundo};
use macroquad::math::DVec2;

// Tipos de objeto (índice en data.js).
const FLECHA: usize = 0;
const PUNTO: usize = 3;
const PELOTA: usize = 4;
const POSTE_IZQUIERDO: usize = 5;
const POSTE_DERECHO: usize = 6;
const TEXTO_GOLES: usize = 8;
const CUADRO_GOL: usize = 9;
const CARTEL_INSTRUCCIONES: usize = 10;
const ARQUERO: usize = 11;
const ICONO_INTENTO: usize = 12;
const ICONO_FALLO: usize = 13;
const BOTON_EMPEZAR: usize = 15;
const TEXTO_EMPEZAR: usize = 16;
const TEXTO_INSTRUCCIONES: usize = 18;
const BOTON_OK: usize = 19;
const BOTON_REINTENTAR: usize = 21;
const TEXTO_RESULTADO: usize = 23;
const BARRA: usize = 27;
const FONDO_GOLES: usize = 28;
const TEXTO_OK: usize = 29;
const ANIMACION_GOL: usize = 30;
const JUGADOR: usize = 31;
const AFUERA: usize = 32;
const POSICION_ARQUERO: usize = 33;
const FONDO: usize = 34;
const CORTINA: usize = 37;
const LOGO: usize = 38;

/// Capa "GUI" del layout "Juego".
const CAPA_GUI: usize = 1;

/// Las variables globales de las hojas, con sus valores iniciales (los que restaura
/// System.ResetGlobals).
struct Globales {
    // Hoja "Juego".
    pelota_activa: f64,
    angulo_tiro: f64,
    salio_afuera: f64,
    atajo_arco: f64,
    atajo_arquero: f64,
    direccion_arquero: String,
    velocidad_arquero: f64,
    perdio: f64,
    anoto: f64,
    scale: f64,
    intentos: f64,
    cantidad_goles: f64,
    velocidad_flecha: f64,
    comenzo: f64,
    // Hoja "Menu Principal".
    cerrar_cortina: f64,
}

impl Default for Globales {
    fn default() -> Self {
        Self {
            pelota_activa: 1.0,
            angulo_tiro: 0.0,
            salio_afuera: 0.0,
            atajo_arco: 0.0,
            atajo_arquero: 0.0,
            direccion_arquero: "ninguna".to_owned(),
            velocidad_arquero: 3.0,
            perdio: 0.0,
            anoto: 0.0,
            scale: 0.6,
            intentos: 3.0,
            cantidad_goles: 0.0,
            velocidad_flecha: 5.0,
            comenzo: 0.0,
            cerrar_cortina: 0.0,
        }
    }
}

/// Cómo terminó un tiro fallado (la variable que se pone a 1 tras la espera).
#[derive(Clone, Copy)]
enum Fallo {
    SalioAfuera,
    AtajoArco,
    AtajoArquero,
}

/// Las acciones que siguen a un System.Wait.
enum Seguir {
    /// Grupo "Choque Pelota", tras Wait(3): la pelota vuelve al punto penal y se marca el fallo.
    Choque(Fallo),
    /// Grupo "Game Over", tras Wait(2): se destruye la pelota, se calla todo y se va al final.
    FinDePartida,
}

pub struct Juego {
    mundo: Mundo,
    audio: Audio,
    g: Globales,
    esperas: Esperas<Seguir>,
    siguiente_layout: Option<String>,
    /// System.IsOnMobile.
    movil: bool,
    /// Posición del ratón (plugin Mouse).
    raton: DVec2,
    cursor: Cursor,
    // Estado de las condiciones "On collision with another object" y "Every X seconds".
    choque_afuera: MemoriaColision,
    choque_poste_derecho: MemoriaColision,
    choque_poste_izquierdo: MemoriaColision,
    choque_arquero: MemoriaColision,
    cortina_cierra: Cada,
    cortina_abre: Cada,
}

impl Juego {
    pub fn nuevo(mundo: Mundo, audio: Audio, movil: bool) -> Self {
        let primero = mundo.proyecto.primer_layout.clone();
        let mut juego = Self {
            mundo,
            audio,
            g: Globales::default(),
            esperas: Esperas::default(),
            siguiente_layout: None,
            movil,
            raton: DVec2::ZERO,
            cursor: Cursor::default(),
            choque_afuera: MemoriaColision::default(),
            choque_poste_derecho: MemoriaColision::default(),
            choque_poste_izquierdo: MemoriaColision::default(),
            choque_arquero: MemoriaColision::default(),
            cortina_cierra: Cada::default(),
            cortina_abre: Cada::default(),
        };
        juego.cambiar_layout(&primero);
        juego
    }

    // --- acciones comunes ----------------------------------------------------------------------

    /// System.GoToLayout: solo cuenta el primero de cada tick.
    fn ir_a(&mut self, layout: &str) {
        if self.siguiente_layout.is_none() {
            self.siguiente_layout = Some(layout.to_owned());
        }
    }

    /// Cambio de layout y "On start of layout". Reiniciar el mismo layout borraría las esperas.
    fn cambiar_layout(&mut self, nombre: &str) {
        if self.mundo.cambiar_layout(nombre) {
            self.esperas.vaciar();
        }
        self.al_empezar_layout();
    }

    /// Las instancias de Pelota cuya velocidad (Bullet) cumple la comparación.
    fn pelotas_con_velocidad(&self, cumple: impl Fn(f64) -> bool) -> Vec<usize> {
        self.mundo
            .de(PELOTA)
            .into_iter()
            .filter(|&i| {
                self.mundo
                    .sprite(i)
                    .bala()
                    .is_some_and(|b| cumple(b.velocidad()))
            })
            .collect()
    }

    /// CustomMovement.SetSpeed(horizontal) en todas las instancias de un tipo.
    fn set_dx(&mut self, i: usize, v: f64) {
        if let Some(m) = self.mundo.sprite_mut(i).movimiento_mut() {
            m.set_velocidad(1, v);
        }
    }

    /// CustomMovement.SetEnabled en todas las instancias de un tipo.
    fn movimiento_habilitado(&mut self, tipo: usize, habilitado: bool) {
        for i in self.mundo.de(tipo) {
            if let Some(m) = self.mundo.sprite_mut(i).movimiento_mut() {
                m.habilitado = habilitado;
            }
        }
    }

    fn set_anim_tipo(&mut self, tipo: usize, nombre: &str) {
        for i in self.mundo.de(tipo) {
            self.mundo.set_anim(i, nombre, 1);
        }
    }

    /// Las expresiones de un tipo sin instancias elegidas usan la primera (0 si no hay).
    fn primera_x(&self, tipo: usize) -> f64 {
        self.mundo
            .primera(tipo)
            .map(|i| self.mundo.posicion(i).0)
            .unwrap_or(0.0)
    }

    // --- entrada y tick ------------------------------------------------------------------------

    /// Toques y ratón entre ticks: los disparadores de Touch, como los eventos del DOM (el
    /// Touch del juego también toma el ratón).
    pub fn entrada(&mut self, e: &Entrada) {
        self.raton = e.raton;
        for &p in &e.inicios {
            self.al_tocar(p);
        }
    }

    /// Un tick del runtime con el dt del navegador (construct2::dt_del_runtime).
    pub fn tick(&mut self, dt_real: f64) {
        // Audio "On ended": el navegador lo lanza entre ticks.
        for etiqueta in self.audio.terminados() {
            self.al_terminar_sonido(&etiqueta);
        }
        self.mundo.avanzar(dt_real);
        self.mundo.quitar_destruidas();
        self.correr_esperas();
        self.mundo.quitar_destruidas();
        self.mundo.tick_comportamientos();
        // Ningún evento usa "On animation finished".
        for i in 0..self.mundo.instancias.len() {
            self.mundo.tick_animacion(i);
        }
        if let Some(nombre) = self.siguiente_layout.take() {
            self.cambiar_layout(&nombre);
        }
        self.mundo.quitar_destruidas();
        if self.mundo.en_layout("Juego") {
            self.hoja_juego();
        } else if self.mundo.en_layout("Menu Principal") {
            self.hoja_menu();
        } else {
            self.hoja_game_over();
        }
        self.mundo.olvidar_colisiones();
        self.mundo.ticks += 1;
    }

    /// System.Wait: lo que queda de las acciones, con las instancias que tenía elegidas.
    fn esperar(&mut self, segundos: f64, seguir: Seguir, elegidas: Vec<u32>) {
        let tiempo = self.mundo.tiempo();
        self.esperas.esperar(tiempo, segundos, seguir, elegidas);
    }

    /// Las esperas cumplidas, al principio del tick. Siguen corriendo aunque se haya cambiado
    /// de layout.
    fn correr_esperas(&mut self) {
        for espera in self.esperas.cumplidas(self.mundo.tiempo()) {
            match espera.seguir {
                Seguir::Choque(fallo) => {
                    for &uid in &espera.elegidas {
                        if let Some(i) = self.mundo.por_uid(uid) {
                            self.mundo.set_posicion(i, 200.0, 420.0);
                        }
                    }
                    match fallo {
                        Fallo::SalioAfuera => self.g.salio_afuera = 1.0,
                        Fallo::AtajoArco => self.g.atajo_arco = 1.0,
                        Fallo::AtajoArquero => self.g.atajo_arquero = 1.0,
                    }
                }
                Seguir::FinDePartida => {
                    self.mundo.destruir_tipo(PELOTA);
                    self.audio.detener_todo();
                    self.ir_a("Game Over");
                }
            }
        }
    }

    // --- disparadores ------------------------------------------------------------------------

    fn al_empezar_layout(&mut self) {
        if self.mundo.en_layout("Juego") {
            self.g.scale = 0.6;
            for i in self.mundo.de(PELOTA) {
                self.mundo.set_escala(i, 0.6);
            }
            self.audio.play("publico3", true, 1.0, "publico");
            if self.movil {
                self.mundo.set_visible_tipo(BARRA, false);
            }
        } else if self.mundo.en_layout("Menu Principal") {
            // Audio.Preload de los diez sonidos: aquí ya están cargados.
            self.fondo_en_movimiento();
            if !self.audio.etiqueta_sonando("musica") {
                self.audio.play("musica3", true, 1.0, "musica");
            }
        } else {
            let texto = format!("{} GOLES", numero_a_texto(self.g.cantidad_goles));
            for i in self.mundo.de(TEXTO_RESULTADO) {
                self.mundo.texto_mut(i).texto = texto.clone();
            }
            self.fondo_en_movimiento();
            if !self.audio.etiqueta_sonando("musica") {
                self.audio.play("musica3", true, 1.0, "musica");
            }
        }
    }

    /// El fondo de rayas avanza a 500 px/s hacia la izquierda (con Wrap).
    fn fondo_en_movimiento(&mut self) {
        self.movimiento_habilitado(FONDO, true);
        for i in self.mundo.de(FONDO) {
            self.set_dx(i, -500.0);
        }
    }

    /// Un toque nuevo: "On touch start" y después "On touched object".
    fn al_tocar(&mut self, p: DVec2) {
        if self.mundo.en_layout("Juego") {
            self.al_tocar_juego(p);
        } else if self.mundo.en_layout("Menu Principal") {
            // Tocar "Empezar" (si se ve): cerrar la cortina.
            let tocados: Vec<usize> = self
                .mundo
                .en_punto(BOTON_EMPEZAR, p)
                .into_iter()
                .filter(|&i| self.mundo.visible(i))
                .collect();
            if !tocados.is_empty()
                && !self.audio.etiqueta_sonando("comenzar")
                && self.g.cerrar_cortina == 0.0
            {
                self.audio.play("click3", false, 1.0, "comenzar");
                self.g.cerrar_cortina = 1.0;
            }
        } else if !self.mundo.en_punto(BOTON_REINTENTAR, p).is_empty()
            && !self.audio.etiqueta_sonando("comenzar")
        {
            // "Reintentar".
            self.audio.play("click3", false, 1.0, "click");
            self.g = Globales::default();
            self.ir_a("Menu Principal");
        }
    }

    fn al_tocar_juego(&mut self, p: DVec2) {
        // Grupo "Lanza Pelota", On touch start: patear si la pelota está quieta en el punto
        // penal.
        if self.g.comenzo == 1.0 {
            let listas: Vec<usize> = self
                .pelotas_con_velocidad(|v| v == 0.0)
                .into_iter()
                .filter(|&i| self.mundo.posicion(i).1 == 420.0)
                .collect();
            if !listas.is_empty() {
                self.mundo.set_visible_tipo(FLECHA, false);
                self.set_anim_tipo(JUGADOR, "pateando");
                self.g.angulo_tiro = self
                    .mundo
                    .primera(FLECHA)
                    .map(|i| grados_de(self.mundo.sprite(i).angulo))
                    .unwrap_or(0.0);
                self.audio.play("tiro3", false, 1.0, "tiro");
            }
        }
        // Grupo "Comenzar", On touched object: el botón OK de las instrucciones.
        if !self.mundo.en_punto(BOTON_OK, p).is_empty()
            && self.g.comenzo == 0.0
            && !self.audio.etiqueta_sonando("comenzar")
        {
            self.audio.play("click3", false, 1.0, "click");
            self.g.comenzo = 1.0;
            let v = self.g.velocidad_flecha;
            for i in self.mundo.de(PUNTO) {
                self.set_dx(i, -v);
            }
            let v = self.g.velocidad_arquero;
            for i in self.mundo.de(POSICION_ARQUERO) {
                self.set_dx(i, -v);
            }
            for tipo in [CARTEL_INSTRUCCIONES, BOTON_OK, TEXTO_INSTRUCCIONES] {
                self.mundo.destruir_tipo(tipo);
            }
            self.g.cantidad_goles = 0.0;
            for x in [70.0, 100.0, 130.0] {
                self.mundo.crear(ICONO_INTENTO, CAPA_GUI, x, 20.0);
            }
            for tipo in [BARRA, TEXTO_GOLES, FONDO_GOLES] {
                self.mundo.set_visible_tipo(tipo, true);
            }
            self.mundo.destruir_tipo(TEXTO_OK);
            self.mundo.set_visible_tipo(FLECHA, true);
        }
    }

    /// Audio "On ended": en el grupo "Choque Pelota", el final del sonido del gol prepara el
    /// siguiente tiro (es una de las condiciones del bloque O).
    fn al_terminar_sonido(&mut self, etiqueta: &str) {
        if self.mundo.en_layout("Juego") && etiqueta.eq_ignore_ascii_case("gol") {
            self.preparar_tiro();
        }
    }

    // --- hoja "Juego" --------------------------------------------------------------------------

    /// Cada grupo tiene un evento de primer nivel ("Is group active"); después de cada uno se
    /// quitan las instancias destruidas.
    fn hoja_juego(&mut self) {
        self.grupo_comenzar();
        self.mundo.quitar_destruidas();
        self.grupo_lanza_pelota();
        self.mundo.quitar_destruidas();
        self.grupo_choque_pelota();
        self.mundo.quitar_destruidas();
        self.grupo_game_over();
        self.mundo.quitar_destruidas();
        self.grupo_movimiento_arquero();
        self.mundo.quitar_destruidas();
    }

    /// Grupo "Comenzar": el cursor de mano sobre el botón OK.
    fn grupo_comenzar(&mut self) {
        let sobre = !self.mundo.en_punto(BOTON_OK, self.raton).is_empty();
        self.cursor.poner(if sobre { 1 } else { 0 });
    }

    /// Grupo "Lanza Pelota": la patada sale en el tercer cuadro de la animación; la pelota se
    /// achica; la flecha apunta al punto que va y viene.
    fn grupo_lanza_pelota(&mut self) {
        if self
            .mundo
            .de(JUGADOR)
            .into_iter()
            .any(|i| self.mundo.sprite(i).cuadro == 2)
        {
            let angulo = self.g.angulo_tiro;
            for i in self.pelotas_con_velocidad(|v| v == 0.0) {
                if let Some(b) = self.mundo.sprite_mut(i).bala_mut() {
                    b.set_velocidad(600.0);
                    b.set_angulo_movimiento(angulo);
                }
            }
        }
        let lanzadas = self.pelotas_con_velocidad(|v| v > 0.0);
        if !lanzadas.is_empty() && self.g.scale > 0.5 {
            self.g.scale -= 0.18;
            for i in lanzadas {
                self.mundo.set_escala(i, self.g.scale);
            }
        }
        // Every tick.
        if !self.mundo.de(FLECHA).is_empty() {
            self.mundo.agregar_objetivo(FLECHA, PUNTO);
        }
        let v = self.g.velocidad_flecha;
        for i in self.mundo.de(PUNTO) {
            if self.mundo.posicion(i).0 < 197.0 {
                self.set_dx(i, v);
            }
        }
        for i in self.mundo.de(PUNTO) {
            if self.mundo.posicion(i).0 > 203.0 {
                self.set_dx(i, -v);
            }
        }
    }

    /// El tiro falló: sonidos, un intento menos, el jugador lamentándose y, 3 segundos después,
    /// la pelota de vuelta en el punto penal.
    fn fallo(&mut self, pelota: usize, fallo: Fallo) {
        self.audio.play("perdio3", false, 1.0, "perdio");
        self.audio.play("seloperdio3", false, 1.0, "seloperdio");
        if !matches!(fallo, Fallo::SalioAfuera) {
            self.g.pelota_activa = 0.0;
        }
        self.g.intentos -= 1.0;
        self.g.perdio = 1.0;
        self.set_anim_tipo(JUGADOR, "perdio");
        let uid = self.mundo.uid(pelota);
        self.esperar(3.0, Seguir::Choque(fallo), vec![uid]);
    }

    /// Grupo "Choque Pelota": afuera, los postes, el arquero y el gol.
    fn grupo_choque_pelota(&mut self) {
        let pelotas = self.mundo.de(PELOTA);
        // On collision with Afuera: el resto del evento corre una vez por cada choque nuevo. No
        // pone pelotaActiva a 0.
        for (p, _) in self.choque_afuera.nuevas(&self.mundo, &pelotas, AFUERA) {
            if self.g.pelota_activa == 1.0 && self.g.salio_afuera == 0.0 {
                self.fallo(p, Fallo::SalioAfuera);
            }
        }
        // Bloque O: On collision with PosteDerecho, On collision with PosteIzquierdo.
        let mut postes = self
            .choque_poste_derecho
            .nuevas(&self.mundo, &pelotas, POSTE_DERECHO);
        postes.extend(
            self.choque_poste_izquierdo
                .nuevas(&self.mundo, &pelotas, POSTE_IZQUIERDO),
        );
        for (p, _) in postes {
            if self.g.pelota_activa == 1.0
                && self.g.atajo_arquero == 0.0
                && self.g.atajo_arco == 0.0
            {
                self.fallo(p, Fallo::AtajoArco);
            }
        }
        for (p, _) in self.choque_arquero.nuevas(&self.mundo, &pelotas, ARQUERO) {
            if self.g.pelota_activa == 1.0
                && self.g.atajo_arco == 0.0
                && self.g.atajo_arquero == 0.0
            {
                self.fallo(p, Fallo::AtajoArquero);
            }
        }
        self.gol();
        // Bloque O: salioAfuera = 1, atajoArco = 1, atajoArquero = 1 (o el final del sonido del
        // gol, como disparador).
        if self.g.salio_afuera == 1.0 || self.g.atajo_arco == 1.0 || self.g.atajo_arquero == 1.0 {
            self.preparar_tiro();
        }
    }

    /// La pelota entera dentro del cuadro del arco (con las expresiones de la primera instancia
    /// de cada tipo).
    fn gol(&mut self) {
        let caja = |tipo: usize| {
            self.mundo.primera(tipo).map(|i| {
                let s = self.mundo.sprite(i);
                (s.x, s.y, s.ancho, s.alto)
            })
        };
        let (cx, cy, cw, ch) = caja(CUADRO_GOL).unwrap_or_default();
        let (px, py, pw, ph) = caja(PELOTA).unwrap_or_default();
        let dentro = cx <= px - pw / 2.0
            && cy <= py - ph / 2.0
            && cx + cw >= px + pw / 2.0
            && cy + ch >= py + ph / 2.0;
        if !(dentro
            && self.g.anoto == 0.0
            && self.g.atajo_arco == 0.0
            && self.g.atajo_arquero == 0.0
            && self.g.pelota_activa == 1.0)
        {
            return;
        }
        self.audio.play("gol3", false, 1.0, "gol");
        self.g.cantidad_goles += 1.0;
        let texto = format!("GOLES: {}", numero_a_texto(self.g.cantidad_goles));
        for i in self.mundo.de(TEXTO_GOLES) {
            self.mundo.texto_mut(i).texto = texto.clone();
        }
        for i in self.mundo.de(PELOTA) {
            if let Some(b) = self.mundo.sprite_mut(i).bala_mut() {
                b.set_velocidad(0.0);
            }
        }
        self.g.anoto = 1.0;
        for i in self.mundo.de(ANIMACION_GOL) {
            self.mundo.set_posicion(i, 200.0, 250.0);
        }
        self.set_anim_tipo(ANIMACION_GOL, "gol");
        self.set_anim_tipo(JUGADOR, "gano");
        if self.g.velocidad_arquero < 25.0 {
            self.g.velocidad_arquero += 5.0;
        }
        if self.g.velocidad_flecha < 15.0 {
            self.g.velocidad_flecha += 0.5;
        }
    }

    /// Todo vuelve a su sitio para el siguiente tiro.
    fn preparar_tiro(&mut self) {
        for i in self.mundo.de(PELOTA) {
            if let Some(b) = self.mundo.sprite_mut(i).bala_mut() {
                b.set_velocidad(0.0);
            }
        }
        self.audio.detener("perdio");
        self.audio.detener("seloperdio");
        for i in self.mundo.de(PELOTA) {
            self.mundo.set_posicion(i, 200.0, 420.0);
        }
        for i in self.mundo.de(ARQUERO) {
            self.mundo.set_posicion(i, 200.0, 127.0);
        }
        self.movimiento_habilitado(POSICION_ARQUERO, true);
        for i in self.mundo.de(ANIMACION_GOL) {
            let x = self.mundo.posicion(i).0;
            self.mundo.set_posicion(i, x, -114.0);
        }
        self.set_anim_tipo(ARQUERO, "Default");
        self.g.direccion_arquero = "ninguna".to_owned();
        self.g.pelota_activa = 1.0;
        self.mundo.set_visible_tipo(FLECHA, true);
        self.g.scale = 0.6;
        for i in self.mundo.de(PELOTA) {
            self.mundo.set_escala(i, 0.6);
        }
        self.g.anoto = 0.0;
        self.g.atajo_arquero = 0.0;
        self.g.atajo_arco = 0.0;
        self.g.salio_afuera = 0.0;
        self.set_anim_tipo(JUGADOR, "parado");
    }

    /// Grupo "Game Over": una cruz por cada intento perdido; con el tercero, 2 segundos después,
    /// el final.
    fn grupo_game_over(&mut self) {
        if self.g.perdio != 1.0 {
            return;
        }
        if self.g.intentos == 2.0 {
            self.mundo.crear(ICONO_FALLO, CAPA_GUI, 130.0, 20.0);
            self.g.perdio = 0.0;
        }
        if self.g.intentos == 1.0 {
            self.mundo.crear(ICONO_FALLO, CAPA_GUI, 100.0, 20.0);
            self.g.perdio = 0.0;
        }
        if self.g.intentos == 0.0 {
            self.mundo.crear(ICONO_FALLO, CAPA_GUI, 70.0, 20.0);
            self.g.perdio = 0.0;
            self.g.anoto = 0.0;
            self.g.intentos = 3.0;
            self.g.velocidad_flecha = 5.0;
            self.g.comenzo = 0.0;
            self.mundo.destruir_tipo(FLECHA);
            self.esperar(2.0, Seguir::FinDePartida, Vec::new());
        }
    }

    /// Grupo "Movimiento Arquero": el arquero se tira hacia donde iba su marca invisible cuando
    /// salió el tiro (o se agacha si estaba en el centro).
    fn grupo_movimiento_arquero(&mut self) {
        self.lanzarse("izquierda", |x| x > 75.0, -3.0, 0);
        self.lanzarse("derecha", |x| x < 320.0, 3.0, 1);
        if self.g.direccion_arquero == "abajo" {
            for i in self.mundo.de(ARQUERO) {
                if !self.mundo.anim_es(i, "abajo") {
                    self.mundo.set_anim(i, "abajo", 1);
                }
            }
        }
        if !self.pelotas_con_velocidad(|v| v > 0.0).is_empty() {
            let marca: Vec<usize> = self.mundo.de(POSICION_ARQUERO);
            let en_centro = marca.iter().any(|&i| {
                let x = self.mundo.posicion(i).0;
                (197.0..=203.0).contains(&x)
            });
            if en_centro && self.g.direccion_arquero == "ninguna" {
                self.g.direccion_arquero = "abajo".to_owned();
            }
            let dx = self
                .mundo
                .primera(POSICION_ARQUERO)
                .and_then(|i| self.mundo.sprite(i).movimiento().map(|m| m.dx))
                .unwrap_or(0.0);
            if dx < 0.0 && self.g.direccion_arquero == "ninguna" {
                self.g.direccion_arquero = "izquierda".to_owned();
            }
            if dx > 0.0 && self.g.direccion_arquero == "ninguna" {
                self.g.direccion_arquero = "derecha".to_owned();
            }
        }
        // Every tick: la marca va y viene entre 190 y 210.
        let v = self.g.velocidad_arquero;
        for i in self.mundo.de(POSICION_ARQUERO) {
            if self.mundo.posicion(i).0 < 190.0 {
                self.set_dx(i, v);
            }
        }
        for i in self.mundo.de(POSICION_ARQUERO) {
            if self.mundo.posicion(i).0 > 210.0 {
                self.set_dx(i, -v);
            }
        }
    }

    /// Los dos eventos de cada lado: mientras no llegue al límite, el arquero vuela 3 px por
    /// tick (y sube 0,2 px hasta y = 118); si no (Else), queda tirado en y = 135.
    fn lanzarse(&mut self, direccion: &str, sin_llegar: fn(f64) -> bool, paso: f64, espejo: i32) {
        let lado = self.g.direccion_arquero == direccion;
        let volando: Vec<usize> = if lado {
            self.mundo
                .de(ARQUERO)
                .into_iter()
                .filter(|&i| sin_llegar(self.mundo.posicion(i).0))
                .collect()
        } else {
            Vec::new()
        };
        if !volando.is_empty() {
            for &i in &volando {
                let (x, y) = self.mundo.posicion(i);
                self.mundo.set_posicion(i, x + paso, y);
            }
            self.movimiento_habilitado(POSICION_ARQUERO, false);
            for &i in &volando {
                self.mundo.set_anim(i, "lanzado", 1);
                self.mundo.sprite_mut(i).set_espejo(espejo);
            }
            for &i in &volando {
                let (x, y) = self.mundo.posicion(i);
                if y > 118.0 {
                    self.mundo.set_posicion(i, x, y - 0.2);
                }
            }
        } else if self.g.direccion_arquero == direccion {
            for i in self.mundo.de(ARQUERO) {
                self.mundo.set_anim(i, "tirado", 1);
                let x = self.mundo.posicion(i).0;
                self.mundo.set_posicion(i, x, 135.0);
            }
        }
    }

    // --- hoja "Menu Principal" -----------------------------------------------------------------

    fn hoja_menu(&mut self) {
        let sobre = self
            .mundo
            .en_punto(BOTON_EMPEZAR, self.raton)
            .into_iter()
            .any(|i| self.mundo.visible(i));
        self.cursor.poner(if sobre { 1 } else { 0 });
        // Con la cortina abierta del todo aparece el botón "Empezar".
        let abierta = self
            .mundo
            .de(CORTINA)
            .into_iter()
            .any(|i| self.mundo.sprite(i).opacidad_porcentaje() <= 0.0);
        if abierta {
            for i in self
                .mundo
                .de(TEXTO_EMPEZAR)
                .into_iter()
                .chain(self.mundo.de(BOTON_EMPEZAR))
            {
                self.mundo.set_posicion(i, 200.0, 345.0);
            }
        }
        if self.g.cerrar_cortina == 1.0 {
            // Cada 0,01 s (en la práctica, cada tick) la cortina se cierra un 1 % y la música
            // baja 0,3 dB.
            if self.cortina_cierra.cumple(0.01, self.mundo.tiempo()) {
                self.cambiar_cortina(1.0);
                let v = self.audio.volumen("musica");
                self.audio.set_volumen("musica", v - 0.3);
            }
            let cerrada = self
                .mundo
                .de(CORTINA)
                .into_iter()
                .any(|i| self.mundo.sprite(i).opacidad_porcentaje() == 100.0);
            if cerrada {
                self.audio.detener("musica");
                self.g.cerrar_cortina = 0.0;
                self.ir_a("Juego");
            }
        } else if self.cortina_abre.cumple(0.01, self.mundo.tiempo()) {
            // Else: la cortina se abre un 1 % por tick.
            self.cambiar_cortina(-1.0);
        }
        // El logo entra desde la derecha, 4 px por tick.
        if self.primera_x(LOGO) > 200.0 {
            for i in self.mundo.de(LOGO) {
                let (x, y) = self.mundo.posicion(i);
                self.mundo.set_posicion(i, x - 4.0, y);
            }
        }
    }

    /// Cortina.SetOpacity(Cortina.Opacity + delta).
    fn cambiar_cortina(&mut self, delta: f64) {
        for i in self.mundo.de(CORTINA) {
            let s = self.mundo.sprite_mut(i);
            let o = s.opacidad_porcentaje();
            s.set_opacidad(o + delta);
        }
    }

    // --- hoja "Game Over" ----------------------------------------------------------------------

    fn hoja_game_over(&mut self) {
        let sobre = !self.mundo.en_punto(BOTON_REINTENTAR, self.raton).is_empty();
        self.cursor.poner(if sobre { 1 } else { 0 });
        // Un evento sin condiciones (era el del botón "Compartir", ver el README): "click" suena
        // en cada tick.
        if !self.audio.etiqueta_sonando("comenzar") {
            self.audio.play("click3", false, 1.0, "click");
        }
    }

    // --- dibujo --------------------------------------------------------------------------------

    pub fn dibujar(&self) {
        self.mundo.dibujar();
    }
}
