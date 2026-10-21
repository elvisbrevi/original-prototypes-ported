// Los scripts de Assembly-CSharp.dll traducidos del C# decompilado (reference/csharp) y el frame
// de Unity 3.4 que los llama:
//
// 1. Application.LoadLevel pendiente: la escena nueva (Start de sus scripts y los AudioSource con
//    Play On Awake).
// 2. Time e Input (los ejes con su suavizado).
// 3. Los pasos de física a 50 Hz, cada uno con sus OnCollisionEnter y OnCollisionStay.
// 4. SendMouseEvents: el rayo de la cámara por el ratón (si la GUI no lo usó en el frame
//    anterior) y OnMouseDown/OnMouseDrag.
// 5. Update de los scripts, en el orden del archivo de la escena, y las corrutinas (iTween).
// 6. Las partículas heredadas (emisión, animación, choques con OnParticleCollision).
// 7. El borrado de lo destruido, el dibujo, el destello de la luz y OnGUI.
//
// Los campos static de los scripts (Bola.posicion, perdio, gano, Activo; Objetivo.fin;
// Transicion._alpha, FadeIn, TransicionRealizada) viven en `Estaticos` y sobreviven a LoadLevel.

use std::collections::{HashMap, HashSet};

use macroquad::prelude::*;
use unity::audio3d::{Audio3d, volumen_3d};
use unity::datos::{EstadoEstilo, Estilo};
use unity::datos3d::ParcheDetalle;
use unity::datos3d::{EstadoPiel, EstiloPiel, Proyecto3d, Value, referencia_objeto};
use unity::entrada::{Entrada, Teclas};
use unity::escena3d::{Escena3d, Id, angulos_euler};
use unity::fisica3d::{Fisica3d, Par};
use unity::gui::{Evento, Gui};
use unity::malla::Malla;
use unity::render3d::{Billboards, Camara3d, CuadroPantalla, Render3d, luz_de};
use unity::terreno::Terreno;
use unity::tiempo::Tiempo;

use crate::itween::RotarPor;

/// Screen.width y Screen.height: la resolución por defecto del build (1024x600, la del netbook
/// en que se hizo, según su output_log.txt).
pub const ANCHO: f32 = 1024.0;
pub const ALTO: f32 = 600.0;

/// LensFlare.fadeSpeed por defecto.
const RAPIDEZ_DESTELLO: f32 = 3.0;
/// El tamaño de los elementos del destello: tamaño / 100 * esto * el alto de la pantalla
/// (calibrado con las capturas del original).
const ESCALA_DESTELLO: f32 = 2.5;

#[derive(Default)]
struct Estaticos {
    /// Bola.posicion.
    posicion: Vec2,
    perdio: bool,
    gano: bool,
    /// Bola.Activo.
    activo: bool,
    /// Objetivo.fin.
    fin: bool,
    /// Transicion._alpha (empieza en 1: la primera escena aparece desde negro).
    alpha: f32,
    fade_in: bool,
    transicion_realizada: bool,
}

struct Bola {
    id: Id,
    limite_min: i32,
    limite_max: i32,
    explosion: Option<Id>,
    velocidad: f32,
    velocidad_salto: f32,
    saltando: bool,
    colision: bool,
    exploto: bool,
}

struct MoverObjeto {
    id: Id,
    screen_space: Vec3,
    offset: Vec3,
    horizontal: bool,
    vertical: bool,
    auto: bool,
    rotacion: bool,
    min_vertical: f32,
    max_vertical: f32,
    min_horizontal: f32,
    max_horizontal: f32,
    velocidad_horizontal: f32,
    velocidad_vertical: f32,
    velocidad_rotacion: f32,
    ida_vertical: bool,
    ida_horizontal: bool,
    ida_rotar: bool,
}

struct Objetivo {
    id: Id,
    sonido_final: Option<Id>,
    tween: RotarPor,
    primera: bool,
}

struct Camara {
    id: Id,
    seguir_y: bool,
    seguir_x: bool,
    menu_rect: Rect,
    salir: bool,
    ir_menu: bool,
    reiniciar: bool,
    repetir: bool,
    window_rect: Rect,
    click: Option<Id>,
    anterior_time: f32,
}

struct Transicion {
    sig_nivel: String,
}

struct EmpezarJuego {
    click: Option<Id>,
}

enum Script {
    Bola(Bola),
    Mover(Box<MoverObjeto>),
    Objetivo(Box<Objetivo>),
    Camara(Camara),
    Transicion(Transicion),
    Empezar(EmpezarJuego),
}

/// La entrada del frame.
#[derive(Default)]
pub struct EntradaFrame<'a> {
    pub teclas_abajo: Vec<&'a str>,
    pub teclas_pulsadas: Vec<&'a str>,
    /// Input.GetMouseButtonDown(0), GetMouseButton(0) y el botón soltado.
    pub pulsado: bool,
    pub apretado: bool,
    pub soltado: bool,
    /// La posición del ratón en la pantalla de 1024x600 (origen arriba), si hay puntero.
    pub raton: Option<Vec2>,
}

/// Lo que el juego le pide a la ventana.
#[derive(Default)]
pub struct Pedidos {
    /// Screen.fullScreen = valor.
    pub pantalla_completa: Option<bool>,
    /// Application.Quit.
    pub salir: bool,
}

pub struct Juego {
    proyecto: Proyecto3d,
    pub escena: Escena3d,
    fisica: Fisica3d,
    pub tiempo: Tiempo,
    entrada: Entrada,
    audio: Audio3d,
    scripts: Vec<Script>,
    est: Estaticos,
    /// Application.LoadLevel pendiente.
    cargar: Option<usize>,
    pares: HashSet<Par>,
    /// SendMouseEvents.m_MouseDownHit (de los colisionadores).
    raton_abajo: Option<Id>,
    /// GUIUtility.mouseUsed: el ratón quedó sobre un control de la GUI.
    raton_usado: bool,
    /// Time.realtimeSinceStartup.
    reloj: f64,
    /// Screen.fullScreen.
    pub pantalla_completa: bool,
    pub pedidos: Pedidos,
    /// El brillo del destello de la luz (0 a 1).
    brillo_destello: f32,
    estilos: HashMap<String, Estilo>,
    /// La textura del GUITexture de la cámara («pixel»).
    textura_gui: Option<Texture2D>,
    /// La escena cambió y hay que subir (o quitar) su terreno en el próximo dibujo.
    terreno_pendiente: bool,
}

fn campo_f32(c: &Value, n: &str) -> f32 {
    c[n].as_f64().unwrap_or(0.0) as f32
}

fn campo_bool(c: &Value, n: &str) -> bool {
    c[n].as_i64().unwrap_or(0) != 0 || c[n].as_bool().unwrap_or(false)
}

/// Un GUIStyle de la piel con la forma que usa la GUI del motor.
fn estilo_de(e: &EstiloPiel) -> Estilo {
    let estado = |s: &EstadoPiel| EstadoEstilo {
        fondo: s.fondo.clone(),
        color: s.color,
    };
    let f = |x: [i32; 4]| x.map(|v| v as f32);
    Estilo {
        normal: estado(&e.normal),
        hover: estado(&e.hover),
        active: estado(&e.active),
        on_normal: Some(estado(&e.on_normal)),
        on_hover: Some(estado(&e.on_hover)),
        on_active: Some(estado(&e.on_active)),
        borde: f(e.borde),
        relleno: f(e.relleno),
        fuente: e.fuente.clone(),
        tamano: e.tamano,
        estilo: e.estilo,
        alineacion: e.alineacion,
        ajuste: e.ajuste,
        recorte: e.recorte,
        desplazamiento: e.desplazamiento,
    }
}

impl Juego {
    pub fn nuevo(
        proyecto: Proyecto3d,
        mallas: &HashMap<String, Malla>,
        audio: Audio3d,
        textura_gui: Option<Texture2D>,
    ) -> Self {
        let fisica = Fisica3d::nueva(&proyecto.fisica, mallas);
        let tiempo = Tiempo::nuevo(&proyecto.tiempo);
        let entrada = Entrada::nueva(&proyecto.ejes);
        let estilos = proyecto
            .piel
            .estilos
            .iter()
            .map(|(k, e)| (k.clone(), estilo_de(e)))
            .collect();
        let escena = Escena3d::cargar(&proyecto, 0);
        let mut juego = Self {
            fisica,
            tiempo,
            entrada,
            audio,
            scripts: Vec::new(),
            est: Estaticos {
                alpha: 1.0,
                ..Default::default()
            },
            cargar: None,
            pares: HashSet::new(),
            raton_abajo: None,
            raton_usado: false,
            reloj: 0.0,
            pantalla_completa: false,
            pedidos: Pedidos::default(),
            brillo_destello: 0.0,
            estilos,
            textura_gui,
            terreno_pendiente: true,
            escena,
            proyecto,
        };
        juego.iniciar_escena();
        juego
    }

    fn estilo(&self, nombre: &str) -> Estilo {
        self.estilos[nombre].clone()
    }

    /// Los scripts de la escena recién cargada, su Start y los AudioSource con Play On Awake.
    fn iniciar_escena(&mut self) {
        self.terreno_pendiente = true;
        self.audio.parar_todo();
        self.pares.clear();
        self.raton_abajo = None;
        self.brillo_destello = 0.0;
        let original =
            |escena: &Escena3d, v: &Value| referencia_objeto(v).and_then(|o| escena.original(o));
        let mut scripts = Vec::new();
        for id in self.escena.ids() {
            for s in &self.escena.objeto(id).scripts {
                let c = &s.campos;
                scripts.push(match s.nombre.as_str() {
                    "Bola" => Script::Bola(Bola {
                        id,
                        limite_min: c["limiteMin"].as_i64().unwrap_or(0) as i32,
                        limite_max: c["limiteMax"].as_i64().unwrap_or(0) as i32,
                        explosion: original(&self.escena, &c["explosion"]),
                        velocidad: 2.0,
                        velocidad_salto: 4.0,
                        saltando: false,
                        colision: false,
                        exploto: false,
                    }),
                    "MoverObjeto" => Script::Mover(Box::new(MoverObjeto {
                        id,
                        screen_space: Vec3::ZERO,
                        offset: Vec3::ZERO,
                        horizontal: campo_bool(c, "Horizontal"),
                        vertical: campo_bool(c, "Vertical"),
                        auto: campo_bool(c, "Auto"),
                        rotacion: campo_bool(c, "Rotacion"),
                        min_vertical: campo_f32(c, "MinVertical"),
                        max_vertical: campo_f32(c, "MaxVertical"),
                        min_horizontal: campo_f32(c, "MinHorizontal"),
                        max_horizontal: campo_f32(c, "MaxHorizontal"),
                        velocidad_horizontal: campo_f32(c, "VelocidadHorizontal"),
                        velocidad_vertical: campo_f32(c, "VelocidadVertical"),
                        velocidad_rotacion: campo_f32(c, "VelocidadRotacion"),
                        ida_vertical: campo_bool(c, "idaVertical"),
                        ida_horizontal: campo_bool(c, "idaHorizontal"),
                        ida_rotar: campo_bool(c, "idaRotar"),
                    })),
                    "Objetivo" => Script::Objetivo(Box::new(Objetivo {
                        id,
                        sonido_final: original(&self.escena, &c["sonidoFinal"]),
                        // Start: iTween.RotateBy(gameObject, Hash("x", 0.25, "easeType",
                        // "easeInOutBack", "loopType", "pingPong", "delay", 0.4)).
                        tween: RotarPor::nuevo(0.25, 0.4),
                        primera: true,
                    })),
                    "Camara" => Script::Camara(Camara {
                        id,
                        seguir_y: campo_bool(c, "SeguirY"),
                        seguir_x: campo_bool(c, "SeguirX"),
                        // Start (y Application.isWebPlayer es false: el build es de escritorio).
                        menu_rect: Rect::new(ANCHO - 80.0, 0.0, 80.0, 20.0),
                        salir: false,
                        ir_menu: false,
                        reiniciar: false,
                        repetir: false,
                        window_rect: Rect::new(ANCHO / 2.0 - 125.0, ALTO / 2.0, 250.0, 50.0),
                        click: original(&self.escena, &c["click"]),
                        anterior_time: 0.0,
                    }),
                    "Transicion" => {
                        // Start.
                        self.est.perdio = false;
                        self.est.gano = false;
                        self.est.fin = false;
                        self.est.transicion_realizada = false;
                        self.est.fade_in = false;
                        Script::Transicion(Transicion {
                            sig_nivel: c["sigNivel"].as_str().unwrap_or("").to_owned(),
                        })
                    }
                    "EmpezarJuego" => Script::Empezar(EmpezarJuego {
                        click: original(&self.escena, &c["click"]),
                    }),
                    otro => panic!("script sin traducir: {otro}"),
                });
            }
        }
        // Bola.Start.
        if scripts.iter().any(|s| matches!(s, Script::Bola(_))) {
            self.est.activo = true;
        }
        self.scripts = scripts;
        for id in self.escena.ids() {
            for i in 0..self.escena.objeto(id).audio.len() {
                if self.escena.objeto(id).audio[i].al_despertar {
                    self.play(id, i);
                }
            }
        }
    }

    // --- motor -----------------------------------------------------------------------------------

    /// La Camera de la escena (la de Main Camera).
    fn camara(&self) -> Option<(Id, Camara3d)> {
        self.escena.ids().into_iter().find_map(|id| {
            let c = self.escena.objeto(id).camara.as_ref()?;
            Some((
                id,
                Camara3d {
                    posicion: self.escena.posicion(id),
                    rotacion: self.escena.rotacion(id),
                    campo_vision: c.campo_vision,
                    cerca: c.cerca,
                    lejos: c.lejos,
                    ancho: ANCHO,
                    alto: ALTO,
                },
            ))
        })
    }

    /// AudioSource.Play del i-ésimo AudioSource de un objeto, atenuado por la distancia a la
    /// AudioListener (la cámara) si el clip es 3D.
    fn play(&mut self, id: Id, i: usize) {
        if !self.escena.existe(id) {
            return;
        }
        let f = self.escena.objeto(id).audio[i].clone();
        let tres_d = f
            .clip
            .as_ref()
            .and_then(|c| self.proyecto.audio.get(c))
            .is_some_and(|c| c.tres_d);
        let volumen = match self.camara() {
            Some((_, c)) if tres_d => {
                volumen_3d(&f, (self.escena.posicion(id) - c.posicion).length())
            }
            _ if f.mudo => 0.0,
            _ => f.volumen,
        };
        self.audio.play(&f, volumen, self.reloj);
    }

    /// AudioSource.isPlaying.
    fn suena(&self, id: Option<Id>) -> bool {
        id.filter(|&i| self.escena.existe(i))
            .and_then(|i| self.escena.objeto(i).audio.first())
            .is_some_and(|f| self.audio.suena(f, self.reloj))
    }

    /// `if (!click.isPlaying) click.Play();`
    fn click(&mut self, id: Option<Id>) {
        if let Some(i) = id
            && !self.suena(id)
        {
            self.play(i, 0);
        }
    }

    /// Application.LoadLevel(nombre): la escena se carga al empezar el frame siguiente.
    fn load_level(&mut self, nombre: &str) {
        if let Some(n) = self.proyecto.nivel(nombre) {
            self.cargar = Some(n);
        }
    }

    /// Object.Destroy: además, los AudioSource del objeto y sus hijos dejan de sonar.
    fn destruir(&mut self, id: Id) {
        let mut pila = vec![id];
        while let Some(i) = pila.pop() {
            if !self.escena.existe(i) {
                continue;
            }
            for f in self.escena.objeto(i).audio.clone() {
                self.audio.stop(&f);
            }
            pila.extend(self.escena.objeto(i).hijos.iter().copied());
        }
        self.escena.destruir(id);
    }

    fn emitiendo(&self, id: Id) -> bool {
        self.escena.existe(id)
            && self
                .escena
                .objeto(id)
                .particulas
                .as_ref()
                .is_some_and(|p| p.emitir)
    }

    fn emitir(&mut self, id: Id) {
        if let Some(p) = self.escena.objeto_mut(id).particulas.as_mut() {
            p.emitir = true;
        }
    }

    fn cantidad_particulas(&self, id: Id) -> usize {
        self.escena
            .objeto(id)
            .particulas
            .as_ref()
            .map_or(0, |p| p.cantidad())
    }

    fn renderer_activo(&self, id: Id) -> bool {
        self.escena
            .objeto(id)
            .renderer
            .as_ref()
            .is_some_and(|r| r.activo)
    }

    fn ocultar(&mut self, id: Id) {
        if let Some(r) = self.escena.objeto_mut(id).renderer.as_mut() {
            r.activo = false;
        }
    }

    // --- frame -----------------------------------------------------------------------------------

    pub fn frame(&mut self, real: f32, entrada: &EntradaFrame) {
        if let Some(n) = self.cargar.take() {
            self.escena = Escena3d::cargar(&self.proyecto, n);
            self.iniciar_escena();
        }
        self.reloj += real as f64;
        self.tiempo.nuevo_frame(real);
        self.entrada.actualizar(
            &Teclas {
                abajo: &entrada.teclas_abajo,
                pulsadas: &entrada.teclas_pulsadas,
            },
            real,
        );
        while self.tiempo.siguiente_paso_fijo() {
            let dt = self.tiempo.paso_fijo;
            self.fisica.paso(&mut self.escena, dt);
            let nuevos: HashSet<Par> = self.fisica.pares.iter().copied().collect();
            let mut pares: Vec<Par> = nuevos.iter().copied().collect();
            pares.sort_by_key(|p| (p.cuerpo, p.otro));
            for p in pares {
                if !self.escena.existe(p.cuerpo) || !self.escena.existe(p.otro) {
                    continue;
                }
                if self.pares.contains(&p) {
                    self.on_collision_stay(p.cuerpo, p.otro);
                    self.on_collision_stay(p.otro, p.cuerpo);
                } else {
                    self.on_collision_enter(p.cuerpo, p.otro);
                    self.on_collision_enter(p.otro, p.cuerpo);
                }
            }
            self.pares = nuevos;
        }
        self.eventos_raton(entrada);
        self.update();
        let dt = self.tiempo.delta;
        for s in &mut self.scripts {
            if let Script::Objetivo(o) = s
                && self.escena.existe(o.id)
            {
                let angulos = angulos_euler(self.escena.rotacion(o.id));
                o.tween.corrutina(dt, angulos, &mut o.primera);
            }
        }
        self.particulas(dt);
        self.escena.limpiar_destruidos();
    }

    /// SendMouseEvents.DoSendMouseEvents (solo la parte de los colisionadores: el GUITexture de la
    /// cámara queda fuera de la pantalla).
    fn eventos_raton(&mut self, entrada: &EntradaFrame) {
        let mut objetivo = None;
        if !self.raton_usado
            && let (Some(raton), Some((_, camara))) = (entrada.raton, self.camara())
        {
            self.fisica.sincronizar(&mut self.escena);
            let (origen, dir) = camara.rayo(vec2(raton.x, ALTO - raton.y));
            objetivo = self
                .fisica
                .rayo(&self.escena, origen, dir, camara.lejos, None)
                .map(|i| i.objeto);
        }
        self.raton_usado = false;
        if entrada.pulsado {
            if let Some(o) = objetivo {
                self.raton_abajo = Some(o);
                self.on_mouse_down(o, entrada);
            }
        } else if !entrada.apretado {
            self.raton_abajo = None;
        } else if let Some(o) = self.raton_abajo
            && self.escena.existe(o)
        {
            self.on_mouse_drag(o, entrada);
        }
    }

    fn update(&mut self) {
        let mut scripts = std::mem::take(&mut self.scripts);
        for s in &mut scripts {
            match s {
                Script::Bola(b) => self.bola_update(b),
                Script::Mover(m) => self.mover_update(m),
                Script::Objetivo(o) => self.objetivo_update(o),
                Script::Camara(c) => self.camara_update(c),
                Script::Transicion(t) => self.transicion_update(t),
                Script::Empezar(_) => {}
            }
        }
        self.scripts = scripts;
    }

    /// Las partículas heredadas de cada objeto, con OnParticleCollision.
    fn particulas(&mut self, dt: f32) {
        self.fisica.sincronizar(&mut self.escena);
        let mut mensajes = Vec::new();
        for id in self.escena.ids() {
            let Some(mut sistema) = self.escena.objeto_mut(id).particulas.take() else {
                continue;
            };
            let matriz = self.escena.matriz_mundo(id);
            let fisica = &self.fisica;
            let escena = &self.escena;
            let mut rayo = |o: Vec3, d: Vec3, l: f32| fisica.rayo(escena, o, d, l, Some(id));
            mensajes.extend(sistema.actualizar(dt, &matriz, &mut rayo));
            self.escena.objeto_mut(id).particulas = Some(sistema);
        }
        for otro in mensajes {
            self.on_particle_collision(otro);
        }
    }

    // --- mensajes ------------------------------------------------------------------------------

    fn on_collision_enter(&mut self, id: Id, otro: Id) {
        let mut scripts = std::mem::take(&mut self.scripts);
        for s in &mut scripts {
            if let Script::Objetivo(o) = s
                && o.id == id
            {
                // Objetivo.OnCollisionEnter.
                if self.escena.objeto(otro).nombre == "Player"
                    && !self.emitiendo(otro)
                    && self.est.activo
                {
                    self.emitir(o.id);
                    self.emitir(otro);
                    self.est.gano = true;
                }
            }
        }
        self.scripts = scripts;
    }

    fn on_collision_stay(&mut self, id: Id, otro: Id) {
        let nombre = self.escena.objeto(otro).nombre.clone();
        let mut scripts = std::mem::take(&mut self.scripts);
        for s in &mut scripts {
            if let Script::Bola(b) = s
                && b.id == id
            {
                // Bola.OnCollisionStay.
                b.colision = true;
                if nombre.contains("Top") {
                    b.saltando = false;
                }
                if nombre == "Sierra" {
                    self.bola_explotar(b);
                }
                if nombre == "Explosivo1"
                    && let Some(c) = self.escena.objeto_mut(b.id).cuerpo.as_mut()
                {
                    c.sumar_impulso(vec3(5.0, 0.0, 0.0));
                }
            }
        }
        self.scripts = scripts;
    }

    fn on_particle_collision(&mut self, id: Id) {
        let mut scripts = std::mem::take(&mut self.scripts);
        for s in &mut scripts {
            if let Script::Bola(b) = s
                && b.id == id
                && self.escena.existe(id)
            {
                self.bola_explotar(b);
            }
        }
        self.scripts = scripts;
    }

    fn on_mouse_down(&mut self, id: Id, entrada: &EntradaFrame) {
        let Some((_, camara)) = self.camara() else {
            return;
        };
        let raton = entrada.raton.map_or(Vec2::ZERO, |r| vec2(r.x, ALTO - r.y));
        for s in &mut self.scripts {
            if let Script::Mover(m) = s
                && m.id == id
            {
                // MoverObjeto.OnMouseDown.
                let pos = self.escena.posicion(m.id);
                m.screen_space = camara.a_pantalla(pos);
                if m.horizontal {
                    m.offset =
                        pos - camara.a_mundo(vec3(raton.x, m.screen_space.y, m.screen_space.z));
                }
                if m.vertical {
                    m.offset =
                        pos - camara.a_mundo(vec3(m.screen_space.x, raton.y, m.screen_space.z));
                }
            }
        }
    }

    fn on_mouse_drag(&mut self, id: Id, entrada: &EntradaFrame) {
        let Some((_, camara)) = self.camara() else {
            return;
        };
        let raton = entrada.raton.map_or(Vec2::ZERO, |r| vec2(r.x, ALTO - r.y));
        let mut scripts = std::mem::take(&mut self.scripts);
        for s in &mut scripts {
            if let Script::Mover(m) = s
                && m.id == id
                && !m.auto
            {
                // MoverObjeto.OnMouseDrag: MoverHorizontalmente y MoverVerticalmente, de 0,05 en
                // 0,05 por frame.
                if m.horizontal {
                    let v = camara.a_mundo(vec3(raton.x, m.screen_space.y, m.screen_space.z))
                        + m.offset;
                    let p = self.escena.posicion(m.id);
                    if p.x < v.x - 0.1 && p.x < m.max_horizontal {
                        self.escena.fijar_posicion(m.id, vec3(p.x + 0.05, p.y, p.z));
                    } else if p.x > v.x + 0.1 && p.x > m.min_horizontal {
                        self.escena.fijar_posicion(m.id, vec3(p.x - 0.05, p.y, p.z));
                    }
                }
                if m.vertical {
                    let v = camara.a_mundo(vec3(m.screen_space.x, raton.y, m.screen_space.z))
                        + m.offset;
                    let p = self.escena.posicion(m.id);
                    if p.y < v.y - 0.1 && p.y < m.max_vertical {
                        self.escena.fijar_posicion(m.id, vec3(p.x, p.y + 0.05, p.z));
                    } else if p.y > v.y + 0.1 && p.y > m.min_vertical {
                        self.escena.fijar_posicion(m.id, vec3(p.x, p.y - 0.05, p.z));
                    }
                }
            }
        }
        self.scripts = scripts;
    }

    // --- Bola ----------------------------------------------------------------------------------

    /// La parte de OnCollisionStay y OnParticleCollision que hace estallar la bola.
    fn bola_explotar(&mut self, b: &mut Bola) {
        if !b.exploto {
            if let Some(e) = b.explosion {
                self.play(e, 0);
            }
            b.exploto = true;
        }
        self.est.activo = false;
        self.emitir(b.id);
    }

    fn bola_update(&mut self, b: &mut Bola) {
        if !self.escena.existe(b.id) || self.escena.objeto(b.id).destruido {
            return;
        }
        let p = self.escena.posicion(b.id);
        self.est.posicion = p.truncate();
        // moverse(): la fuerza va al próximo paso de física (y se acumula si hay varios Update
        // entre dos pasos).
        let h = self.entrada.eje("Horizontal");
        let saltar = self.entrada.boton_pulsado("Saltar");
        if let Some(c) = self.escena.objeto_mut(b.id).cuerpo.as_mut() {
            c.sumar_aceleracion(vec3(h * b.velocidad, 0.0, 0.0));
            if saltar && b.colision && !b.saltando {
                c.velocidad = vec3(c.velocidad.x, b.velocidad_salto, 0.0);
                b.saltando = true;
            }
        }
        // perder().
        if self.cantidad_particulas(b.id) > 1 {
            self.ocultar(b.id);
        }
        let y = self.escena.posicion(b.id).y;
        if y < b.limite_min as f32
            || y > b.limite_max as f32
            || (!self.renderer_activo(b.id) && self.cantidad_particulas(b.id) == 0)
        {
            self.destruir(b.id);
            if !self.est.gano {
                self.est.perdio = true;
            }
        }
    }

    // --- MoverObjeto ---------------------------------------------------------------------------

    fn mover_update(&mut self, m: &mut MoverObjeto) {
        if !m.auto || !self.escena.existe(m.id) {
            return;
        }
        let dt = self.tiempo.delta;
        let id = m.id;
        if m.horizontal {
            let p = self.escena.posicion(id);
            if p.x < m.max_horizontal && m.ida_horizontal {
                self.escena
                    .fijar_posicion(id, vec3(p.x + m.velocidad_horizontal * dt, p.y, p.z));
            } else {
                m.ida_horizontal = false;
            }
            let p = self.escena.posicion(id);
            if p.x > m.min_horizontal && !m.ida_horizontal {
                self.escena
                    .fijar_posicion(id, vec3(p.x - m.velocidad_horizontal * dt, p.y, p.z));
            } else {
                m.ida_horizontal = true;
            }
        }
        if m.vertical {
            let p = self.escena.posicion(id);
            if p.y < m.max_vertical && m.ida_vertical {
                self.escena
                    .fijar_posicion(id, vec3(p.x, p.y + m.velocidad_vertical * dt, p.z));
            } else {
                m.ida_vertical = false;
            }
            let p = self.escena.posicion(id);
            if p.y > m.min_vertical && !m.ida_vertical {
                self.escena
                    .fijar_posicion(id, vec3(p.x, p.y - m.velocidad_vertical * dt, p.z));
            } else {
                m.ida_vertical = true;
            }
        }
        if m.rotacion {
            // Compara la y con los límites horizontales (no con MinRotar/MaxRotar): las sierras,
            // con 0 y 0, giran siempre hacia el mismo lado.
            let y = self.escena.posicion(id).y;
            if y < m.max_horizontal && m.ida_rotar {
                self.escena
                    .girar_local(id, vec3(0.0, 0.0, dt) * m.velocidad_rotacion);
            } else {
                m.ida_rotar = false;
            }
            let y = self.escena.posicion(id).y;
            if y > m.min_horizontal && !m.ida_rotar {
                self.escena
                    .girar_local(id, vec3(0.0, 0.0, dt) * -m.velocidad_rotacion);
            } else {
                m.ida_rotar = true;
            }
        }
    }

    // --- Objetivo ------------------------------------------------------------------------------

    fn objetivo_update(&mut self, o: &mut Objetivo) {
        if !self.escena.existe(o.id) || self.escena.objeto(o.id).destruido {
            return;
        }
        if !self.suena(o.sonido_final)
            && self.est.gano
            && let Some(s) = o.sonido_final
        {
            self.play(s, 0);
        }
        if self.cantidad_particulas(o.id) > 1 {
            self.ocultar(o.id);
        }
        if !self.renderer_activo(o.id) && self.cantidad_particulas(o.id) == 0 {
            self.destruir(o.id);
            self.est.fin = true;
        }
        // iTween.Update (el componente que agregó RotateBy).
        let giro = o.tween.update(self.tiempo.delta);
        if giro != Vec3::ZERO {
            self.escena.girar_local(o.id, giro);
        }
    }

    // --- Camara --------------------------------------------------------------------------------

    fn camara_update(&mut self, c: &mut Camara) {
        if c.ir_menu {
            self.ir_a("menuPrincipal");
        }
        if c.repetir {
            let nombre = self.escena.nombre.clone();
            self.ir_a(&nombre);
        }
        if !self.escena.existe(c.id) {
            return;
        }
        let p = self.escena.posicion(c.id);
        if c.seguir_y && self.est.activo {
            self.escena
                .fijar_posicion(c.id, vec3(p.x, self.est.posicion.y, p.z));
        }
        let p = self.escena.posicion(c.id);
        if c.seguir_x && self.est.activo {
            self.escena
                .fijar_posicion(c.id, vec3(self.est.posicion.x, p.y, p.z));
        }
    }

    // --- Transicion ----------------------------------------------------------------------------

    /// Transicion.irA.
    fn ir_a(&mut self, destino: &str) {
        self.est.fade_in = true;
        if self.est.transicion_realizada {
            self.est.fade_in = false;
            self.load_level(destino);
        }
    }

    fn transicion_update(&mut self, t: &mut Transicion) {
        let dt = self.tiempo.delta;
        if self.est.fin {
            self.est.fade_in = self.est.fin;
            if self.est.transicion_realizada {
                self.est.fade_in = false;
                self.load_level(&t.sig_nivel);
            }
        }
        if self.est.perdio {
            self.est.fade_in = true;
            if self.est.transicion_realizada {
                self.est.fade_in = false;
                let nombre = self.escena.nombre.clone();
                self.load_level(&nombre);
            }
        }
        if self.est.fade_in {
            if self.est.alpha < 1.0 {
                self.est.alpha += dt * 0.5;
            } else {
                self.est.transicion_realizada = true;
            }
        } else if self.est.alpha > 0.0 {
            self.est.alpha -= dt * 0.5;
        } else {
            self.est.transicion_realizada = false;
        }
    }

    // --- dibujo ----------------------------------------------------------------------------------

    /// El dibujo de la escena (la cámara, las partículas y el destello de la luz).
    pub fn dibujar(&mut self, render: &mut Render3d, pase: macroquad::miniquad::RenderPass) {
        let Some((cid, camara)) = self.camara() else {
            return;
        };
        render.tiempo = self.tiempo.tiempo as f32;
        if std::mem::take(&mut self.terreno_pendiente) {
            self.subir_terreno(render, &camara);
        }
        let datos = self.escena.objeto(cid).camara.clone().expect("cámara");
        let ajustes = self.proyecto.escenas[self.escena.nivel].ajustes.clone();
        let a = ajustes.ambiente;
        let luz = luz_de(&self.escena);
        let mut billboards = Vec::new();
        for id in self.escena.ids() {
            let o = self.escena.objeto(id);
            let Some(p) = &o.particulas else {
                continue;
            };
            let Some((true, Some(material), maximo)) = &p.renderer else {
                continue;
            };
            if p.particulas.is_empty() {
                continue;
            }
            billboards.push(Billboards {
                material,
                tamano_maximo: *maximo,
                particulas: p
                    .particulas
                    .iter()
                    .map(|q| (q.posicion, q.tamano, q.rotacion, q.color))
                    .collect(),
            });
        }
        render.dibujar(
            pase,
            &self.escena,
            &camara,
            datos.fondo,
            if datos.borrado == 1 {
                ajustes.cielo.as_deref()
            } else {
                None
            },
            Color::new(a[0], a[1], a[2], a[3]),
            luz.map(|l| l.1),
            &billboards,
        );
        drop(billboards);
        self.destello(render, pase, &camara, ajustes.fuerza_destello);
    }

    /// El Terrain de la escena (el del menú): su malla hasta donde llega la cámara y el pasto hasta
    /// la distancia de los detalles.
    fn subir_terreno(&self, render: &mut Render3d, camara: &Camara3d) {
        render.quitar_terreno();
        let Some(id) = self
            .escena
            .ids()
            .into_iter()
            .find(|&i| self.escena.objeto(i).terreno.is_some())
        else {
            return;
        };
        let campos = self.escena.objeto(id).terreno.clone().unwrap_or_default();
        let alturas = Image::from_file_with_format(
            include_bytes!("../assets/terreno/alturas.png"),
            Some(ImageFormat::Png),
        )
        .expect("alturas");
        let alturas: Vec<u8> = alturas
            .bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0])
            .collect();
        let parches: Vec<ParcheDetalle> =
            serde_json::from_str(include_str!("../assets/terreno/parches.json")).expect("parches");
        let terreno = Terreno::nuevo(&self.proyecto.terreno, alturas, self.escena.posicion(id));
        let distancia = campo_f32(&campos, "distancia_detalles");
        let densidad = campo_f32(&campos, "densidad_detalles");
        render.cargar_terreno(
            &terreno,
            &parches,
            camara.posicion,
            camara.lejos + 4.0,
            distancia,
            densidad,
        );
    }

    /// El LensFlare de la luz (FlareLayer de la cámara): los elementos en la línea que va de la
    /// luz al centro de la pantalla. Se apaga poco a poco si un colisionador tapa la luz.
    fn destello(
        &mut self,
        render: &mut Render3d,
        pase: macroquad::miniquad::RenderPass,
        camara: &Camara3d,
        fuerza: f32,
    ) {
        let Some((lid, luz)) = luz_de(&self.escena) else {
            return;
        };
        let Some(nombre) = self
            .escena
            .objeto(lid)
            .luz
            .as_ref()
            .and_then(|l| l.destello.clone())
        else {
            return;
        };
        let d = &self.proyecto.destello;
        if d.nombre != nombre {
            return;
        }
        let pantalla = camara.a_pantalla(luz.posicion);
        let dentro = pantalla.z > camara.cerca
            && (0.0..=ANCHO).contains(&pantalla.x)
            && (0.0..=ALTO).contains(&pantalla.y);
        let tapada = dentro && {
            self.fisica.sincronizar(&mut self.escena);
            let hacia = luz.posicion - camara.posicion;
            self.fisica
                .rayo(&self.escena, camara.posicion, hacia, hacia.length(), None)
                .is_some()
        };
        let objetivo = if dentro && !tapada { 1.0 } else { 0.0 };
        let paso = RAPIDEZ_DESTELLO * get_frame_time();
        self.brillo_destello = if self.brillo_destello < objetivo {
            (self.brillo_destello + paso).min(objetivo)
        } else {
            (self.brillo_destello - paso).max(objetivo)
        };
        if self.brillo_destello <= 0.0 {
            return;
        }
        let b = self.brillo_destello;
        let origen = vec2(pantalla.x, ALTO - pantalla.y);
        let centro = vec2(ANCHO, ALTO) / 2.0;
        let color_luz = self
            .escena
            .objeto(lid)
            .luz
            .as_ref()
            .map_or(WHITE, |l| l.color);
        let cuadros: Vec<CuadroPantalla> = d
            .elementos
            .iter()
            .map(|e| {
                let pos = origen + (centro - origen) * e.posicion;
                let mut tamano = e.tamano / 100.0 * ESCALA_DESTELLO * ALTO;
                if e.zoom {
                    tamano *= b;
                }
                let mut c = Color::new(e.color[0], e.color[1], e.color[2], e.color[3]);
                if e.color_luz {
                    c = Color::new(c.r * color_luz.r, c.g * color_luz.g, c.b * color_luz.b, c.a);
                }
                let f = fuerza * if e.fundido { b } else { 1.0 };
                let c = Color::new(c.r * f, c.g * f, c.b * f, c.a * f);
                // Rotate: el borde de abajo mira al centro de la pantalla.
                let angulo = if e.girar {
                    let v = centro - pos;
                    v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                };
                // FlareLayout «1 Large 4 Small»: la imagen 0 es la mitad de arriba de la textura y
                // las otras cuatro, los cuartos de la de abajo (en UV de Unity, con v hacia arriba;
                // la textura se dibuja con v = 0 abajo).
                let uv = match e.imagen {
                    0 => [0.0, 0.5, 1.0, 1.0],
                    n => {
                        let i = (n - 1) as f32;
                        let (x, y) = (i % 2.0 * 0.5, 0.25 - (i / 2.0).floor() * 0.25);
                        [x, y, x + 0.5, y + 0.25]
                    }
                };
                CuadroPantalla {
                    centro: pos,
                    tamano,
                    angulo,
                    uv,
                    color: c,
                }
            })
            .collect();
        if let Some(t) = &d.textura {
            render.dibujar_pantalla(pase, t, &cuadros);
        }
    }

    /// OnGUI de los scripts, por cada evento del frame, en el orden de los componentes.
    pub fn gui(&mut self, gui: &mut Gui, entrada: &EntradaFrame) {
        gui.raton = entrada.raton;
        gui.respaldo = true;
        gui.raton_sobre_control = false;
        let mut usado = false;
        for evento in Gui::eventos(entrada.pulsado, entrada.soltado) {
            let mut scripts = std::mem::take(&mut self.scripts);
            for s in &mut scripts {
                gui.empezar(evento);
                match s {
                    Script::Transicion(_) => self.transicion_gui(gui),
                    Script::Camara(c) => self.camara_gui(gui, c),
                    Script::Empezar(e) => self.empezar_gui(gui, e),
                    _ => {}
                }
            }
            self.scripts = scripts;
            if evento == Evento::Repaint {
                usado = gui.raton_sobre_control;
            }
        }
        self.raton_usado = usado;
        gui.presentar();
    }

    fn transicion_gui(&mut self, gui: &mut Gui) {
        gui.color = Color::new(0.0, 0.0, 0.0, self.est.alpha.clamp(0.0, 1.0));
        if let Some(t) = &self.textura_gui {
            gui.dibujar_textura(Rect::new(0.0, 0.0, ANCHO, ALTO), t);
        }
    }

    fn camara_gui(&mut self, gui: &mut Gui, c: &mut Camara) {
        if let Some(t) = &self.textura_gui {
            gui.dibujar_textura(c.menu_rect, t);
        }
        let x = c.menu_rect.x + c.menu_rect.w;
        // botonSalir.
        if gui.button_con_tooltip(
            Rect::new(x - 17.5, 2.5, 15.0, 15.0),
            "",
            "Salir",
            &self.estilo("cerrar"),
        ) {
            self.click(c.click);
            c.salir = true;
        }
        if c.salir {
            let titulo = "¿Desea salir a la pantalla de inicio?";
            c.window_rect = self.ventana(gui, 0, c, titulo, false);
        }
        let (tooltip, completa) = if self.pantalla_completa {
            ("Ventana", false)
        } else {
            ("Maximizar", true)
        };
        if gui.button_con_tooltip(
            Rect::new(x - 35.0, 2.5, 15.0, 15.0),
            "",
            tooltip,
            &self.estilo("maximizar"),
        ) {
            self.click(c.click);
            self.pantalla_completa = completa;
            self.pedidos.pantalla_completa = Some(completa);
        }
        // «Opciones» solo suena.
        if gui.button_con_tooltip(
            Rect::new(x - 55.0, 2.5, 15.0, 15.0),
            "",
            "Opciones",
            &self.estilo("opciones"),
        ) && !self.suena(c.click)
        {
            self.click(c.click);
        }
        // botonReiniciar.
        if gui.button_con_tooltip(
            Rect::new(x - 75.0, 2.5, 15.0, 15.0),
            "",
            "Reiniciar",
            &self.estilo("reiniciar"),
        ) {
            self.click(c.click);
            c.reiniciar = true;
        }
        if c.reiniciar {
            let titulo = "¿Desea volver a iniciarla partida?";
            c.window_rect = self.ventana(gui, 1, c, titulo, true);
        }
        let tooltip = gui.tooltip.clone();
        let r = c.menu_rect;
        gui.label(
            Rect::new(r.x, r.y + r.h + 2.5, r.w, r.h),
            &tooltip,
            &self.estilo("toolTip"),
        );
    }

    /// GUI.Window con VentanaSalir o VentanaReiniciar.
    fn ventana(
        &mut self,
        gui: &mut Gui,
        id: i32,
        c: &mut Camara,
        titulo: &str,
        reiniciar: bool,
    ) -> Rect {
        let ventana = self.estilo("window");
        let boton = self.estilo("button");
        let mut si = false;
        let mut no = false;
        let rect = gui.ventana(id, c.window_rect, titulo, &ventana, |gui| {
            si = gui.button(Rect::new(10.0, 20.0, 100.0, 20.0), "Si", &boton);
            no = gui.button(Rect::new(140.0, 20.0, 100.0, 20.0), "No", &boton);
        });
        // Transicion.fadeIn(0.7f) y la pausa con Time.timeScale.
        self.est.alpha = 0.7;
        if self.tiempo.escala != 0.0 {
            c.anterior_time = self.tiempo.escala;
        }
        self.tiempo.escala = 0.0;
        if si {
            self.click(c.click);
            self.tiempo.escala = c.anterior_time;
            if reiniciar {
                c.repetir = true;
                c.reiniciar = false;
            } else {
                c.ir_menu = true;
                c.salir = false;
            }
        }
        if no {
            self.click(c.click);
            self.tiempo.escala = c.anterior_time;
            if reiniciar {
                c.reiniciar = false;
            } else {
                c.salir = false;
            }
        }
        rect
    }

    fn empezar_gui(&mut self, gui: &mut Gui, e: &mut EmpezarJuego) {
        gui.color = Color::new(1.0, 1.0, 1.0, (1.0 - self.est.alpha).clamp(0.0, 1.0));
        gui.label(
            Rect::new(100.0, 300.0, 10.0, 10.0),
            "Go Ball",
            &self.estilo("titulo"),
        );
        let boton = self.estilo("button");
        if gui.button(Rect::new(100.0, 370.0, 100.0, 20.0), "Jugar", &boton) {
            self.click(e.click);
            self.est.fin = true;
        }
        if gui.button(Rect::new(100.0, 395.0, 100.0, 20.0), "Salir", &boton) {
            self.click(e.click);
            self.pedidos.salir = true;
        }
    }
}
