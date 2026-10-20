// Los scripts de Assembly-CSharp.dll traducidos del C# decompilado (reference/csharp), y el
// frame de Unity que los llama: FixedUpdate y la física 2D a paso fijo, Update, Invoke y las
// corrutinas, el Animator y las partículas, el dibujo y OnGUI. Cada script es una estructura con
// sus campos; los campos static (CameraRunner.x, Spawn.activo, HUD.estadoStatic...) viven en
// `Estaticos`, fuera de la escena, y sobreviven a Application.LoadLevel.

use std::collections::HashMap;

use macroquad::prelude::*;
use museo::Preferencias;
use unity::audio::Audio;
use unity::datos::{Estilo, Proyecto, Value};
use unity::escena::{Escena, Id};
use unity::gui::Gui;
use unity::render::{self, Camara, Recursos};
use unity::tiempo::Tiempo;

/// Screen.width y Screen.height: el tamaño del Web Player (defaultScreenWidthWeb/HeightWeb).
pub const ANCHO: f32 = 960.0;
pub const ALTO: f32 = 600.0;

/// ESTADO_GUI, con los nombres del original.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EstadoGui {
    MainGui,
    InGameGui,
    ShareGui,
}

/// Los campos static de los scripts.
struct Estaticos {
    /// CameraRunner: x = 5, velocidadShake = 0.5 al cargar la clase.
    x: f32,
    max: f32,
    min: f32,
    velocidad_shake: f32,
    ida: bool,
    is_shake: bool,
    /// Spawn.activo = true al cargar la clase.
    spawn_activo: bool,
    /// HUD.estadoStatic.
    estado_static: EstadoGui,
}

impl Default for Estaticos {
    fn default() -> Self {
        Self {
            x: 5.0,
            max: 0.0,
            min: 0.0,
            velocidad_shake: 0.5,
            ida: false,
            is_shake: false,
            spawn_activo: true,
            estado_static: EstadoGui::MainGui,
        }
    }
}

struct CameraRunner {
    id: Id,
    player: Id,
    y: f32,
}

struct Hud {
    id: Id,
    estado: EstadoGui,
    player_score: i32,
    best_player_score: i32,
    new_record: bool,
    window_rect_fin: Rect,
}

/// PlatformerCharacter2D.
struct Personaje {
    id: Id,
    facing_right: bool,
    max_speed: f32,
    jump_force: f32,
    crouch_speed: f32,
    air_control: bool,
    what_is_ground: u32,
    ground_check: Id,
    grounded_radius: f32,
    grounded: bool,
    double_jump: bool,
    can_double_jump: bool,
    polvo: String,
    hit_salto: String,
}

/// Platformer2DUserControl.
struct Control {
    jump: bool,
}

struct Spawn {
    id: Id,
    muro: String,
    spawn_min: f32,
    spawn_max: f32,
    y_min: f32,
    y_max: f32,
    /// Invoke("Spawn_", t): cuándo (en Time.time).
    invoke: Option<f64>,
}

/// CFX_AutoDestructShuriken: la corrutina CheckIfAlive de un objeto.
struct AutoDestruccion {
    id: Id,
    only_deactivate: bool,
    /// Cuándo termina el WaitForSeconds(0.5f) (en Time.time).
    siguiente: f64,
}

/// La entrada del frame.
#[derive(Clone, Copy, Default)]
pub struct Entrada {
    /// Input.GetMouseButtonDown(0).
    pub pulsado: bool,
    /// El botón se soltó en este frame.
    pub soltado: bool,
    /// Event.mousePosition (en la pantalla de 960x600), si hay puntero.
    pub raton: Option<Vec2>,
}

pub struct Juego {
    proyecto: Proyecto,
    pub escena: Escena,
    pub tiempo: Tiempo,
    estaticos: Estaticos,
    audio: Audio,
    preferencias: Preferencias,
    camara: CameraRunner,
    hud: Hud,
    personaje: Personaje,
    control: Control,
    spawn: Spawn,
    /// Destroyer.amount.
    amount: f32,
    /// PowerUp.activo de cada moneda.
    power_ups: HashMap<Id, bool>,
    auto: Vec<AutoDestruccion>,
    /// Application.LoadLevel pendiente: se carga al empezar el frame siguiente.
    cargar: bool,
}

fn campo_f32(campos: &Value, nombre: &str) -> f32 {
    campos[nombre].as_f64().unwrap_or(0.0) as f32
}

fn campo_prefab(campos: &Value, nombre: &str) -> String {
    campos[nombre]["prefab"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

impl Juego {
    pub fn nuevo(proyecto: Proyecto, audio: Audio, preferencias: Preferencias) -> Self {
        let escena = Escena::cargar(&proyecto);
        let tiempo = Tiempo::nuevo(&proyecto.tiempo);
        let mut juego = Self {
            camara: CameraRunner {
                id: 0,
                player: 0,
                y: 0.0,
            },
            hud: Hud {
                id: 0,
                estado: EstadoGui::MainGui,
                player_score: 0,
                best_player_score: 0,
                new_record: false,
                window_rect_fin: Rect::default(),
            },
            personaje: Personaje {
                id: 0,
                facing_right: true,
                max_speed: 0.0,
                jump_force: 0.0,
                crouch_speed: 0.0,
                air_control: false,
                what_is_ground: 0,
                ground_check: 0,
                grounded_radius: 0.2,
                grounded: false,
                double_jump: false,
                can_double_jump: false,
                polvo: String::new(),
                hit_salto: String::new(),
            },
            control: Control { jump: false },
            spawn: Spawn {
                id: 0,
                muro: String::new(),
                spawn_min: 0.0,
                spawn_max: 0.0,
                y_min: 0.0,
                y_max: 0.0,
                invoke: None,
            },
            amount: 0.0,
            power_ups: HashMap::new(),
            auto: Vec::new(),
            cargar: false,
            proyecto,
            escena,
            tiempo,
            estaticos: Estaticos::default(),
            audio,
            preferencias,
        };
        juego.iniciar_escena();
        juego
    }

    fn estilo(&self, nombre: &str) -> Estilo {
        self.proyecto.estilos[nombre].clone()
    }

    fn objeto_con_script(&self, script: &str) -> (Id, Value) {
        self.escena
            .ids()
            .into_iter()
            .find_map(|id| {
                let s = self.escena.objeto(id).script(script)?;
                Some((id, s.campos.clone()))
            })
            .expect("script en la escena")
    }

    /// Los Awake y los Start de la escena recién cargada, en el orden de sus objetos.
    fn iniciar_escena(&mut self) {
        self.power_ups.clear();
        self.auto.clear();
        // PlatformerCharacter2D.Awake y los campos serializados.
        let (id, c) = self.objeto_con_script("PlatformerCharacter2D");
        let ground_check = self
            .escena
            .objeto(id)
            .hijos
            .iter()
            .copied()
            .find(|&h| self.escena.objeto(h).nombre == "GroundCheck")
            .expect("GroundCheck");
        self.personaje = Personaje {
            id,
            facing_right: true,
            max_speed: campo_f32(&c, "maxSpeed"),
            jump_force: campo_f32(&c, "jumpForce"),
            crouch_speed: campo_f32(&c, "crouchSpeed"),
            air_control: c["airControl"].as_i64() == Some(1),
            what_is_ground: c["whatIsGround"].as_u64().unwrap_or(0) as u32,
            ground_check,
            grounded_radius: 0.2,
            grounded: false,
            double_jump: false,
            can_double_jump: c["canDoubleJump"].as_i64() == Some(1),
            polvo: campo_prefab(&c, "polvo"),
            hit_salto: campo_prefab(&c, "hitSalto"),
        };
        self.control = Control { jump: false };
        let (id, c) = self.objeto_con_script("CameraRunner");
        self.camara = CameraRunner {
            id,
            player: self
                .escena
                .original(c["player"]["objeto"].as_i64().expect("player")),
            y: campo_f32(&c, "y"),
        };
        let (id, _) = self.objeto_con_script("HUD");
        self.hud = Hud {
            id,
            estado: EstadoGui::MainGui,
            player_score: 0,
            best_player_score: 0,
            new_record: false,
            window_rect_fin: Rect::default(),
        };
        let (id, c) = self.objeto_con_script("Spawn");
        self.spawn = Spawn {
            id,
            muro: campo_prefab(&c, "muro"),
            spawn_min: campo_f32(&c, "spawnMin"),
            spawn_max: campo_f32(&c, "spawnMax"),
            y_min: campo_f32(&c, "yMin"),
            y_max: campo_f32(&c, "yMax"),
            invoke: None,
        };
        let (_, c) = self.objeto_con_script("Destroyer");
        self.amount = campo_f32(&c, "amount");
        self.escena.sincronizar_fisica();
        // Start, en el orden de los objetos: Main Camera (CameraRunner, HUD) y GroundSpawn.
        self.camera_runner_start();
        self.hud_start();
        self.spawn_();
    }

    /// Los Awake (y OnEnable) de los scripts de un prefab recién instanciado.
    fn despertar(&mut self, raiz: Id) {
        let mut pendientes = vec![raiz];
        while let Some(id) = pendientes.pop() {
            let obj = self.escena.objeto(id);
            pendientes.extend(obj.hijos.iter().copied());
            for s in &obj.scripts {
                match s.nombre.as_str() {
                    "PowerUp" => {
                        self.power_ups.insert(id, true);
                    }
                    // OnEnable: StartCoroutine("CheckIfAlive"), que espera 0,5 s antes de mirar.
                    "CFX_AutoDestructShuriken" => self.auto.push(AutoDestruccion {
                        id,
                        only_deactivate: s.campos["OnlyDeactivate"].as_i64() == Some(1),
                        siguiente: self.tiempo.tiempo + 0.5,
                    }),
                    _ => {}
                }
            }
        }
    }

    fn instanciar(&mut self, prefab: &str, posicion: Option<Vec3>, rotacion: Option<Quat>) -> Id {
        let id = self
            .escena
            .instanciar(&self.proyecto, prefab, posicion, rotacion);
        self.despertar(id);
        id
    }

    // --- CameraRunner -----------------------------------------------------------------------

    fn camera_runner_start(&mut self) {
        let e = &mut self.estaticos;
        e.max = e.x + 1.0;
        e.min = e.x - 1.0;
        e.ida = false;
        e.is_shake = false;
        e.velocidad_shake = 0.5;
        e.x = 4.0;
    }

    fn camera_runner_update(&mut self) {
        let jugador = self.escena.posicion(self.camara.player);
        let x = self.estaticos.x;
        self.escena
            .fijar_posicion(self.camara.id, vec3(jugador.x + x, self.camara.y, -10.0));
        if self.estaticos.is_shake && self.estaticos.velocidad_shake > 0.0 {
            self.shake();
        } else {
            self.estaticos.is_shake = false;
        }
    }

    fn shake(&mut self) {
        let escala = self.tiempo.escala;
        let delta = self.tiempo.delta;
        let e = &mut self.estaticos;
        if e.ida {
            if e.x < e.max {
                e.x += e.velocidad_shake * escala;
            } else {
                e.ida = false;
            }
        } else if e.x > e.min {
            e.x -= e.velocidad_shake * escala;
        } else {
            e.ida = true;
        }
        e.max -= 0.2 * delta;
        e.min += 0.2 * delta;
        e.velocidad_shake -= 0.02 * escala;
    }

    // --- HUD --------------------------------------------------------------------------------

    fn hud_start(&mut self) {
        self.hud.best_player_score = self.preferencias.leer_entero("BestScore", 0);
        self.tiempo.escala = 0.0;
        self.hud.window_rect_fin = Rect::new(
            (ANCHO / 2.0 - 320.0).trunc(),
            (ALTO / 2.0 - 300.0).trunc(),
            640.0,
            480.0,
        );
    }

    fn hud_update(&mut self) {
        self.estaticos.estado_static = self.hud.estado;
    }

    fn increase_score(&mut self, amount: i32) {
        self.hud.player_score += amount;
    }

    fn play(&self, objeto: Id, fuente: usize) {
        self.audio.play(&self.escena.objeto(objeto).audio[fuente]);
    }

    fn stop(&self, objeto: Id, fuente: usize) {
        self.audio.stop(&self.escena.objeto(objeto).audio[fuente]);
    }

    /// HUD.OnGUI, una vez por evento.
    fn on_gui(&mut self, gui: &mut Gui) {
        if !self.estaticos.is_shake && !self.estaticos.spawn_activo {
            self.hud.estado = EstadoGui::ShareGui;
        }
        let boton = Rect::new(ANCHO - 150.0, ALTO - 130.0, 120.0, 100.0);
        match self.hud.estado {
            EstadoGui::MainGui => {
                gui.label(
                    Rect::new(ANCHO / 2.0 - 140.0, 100.0, 280.0, 400.0),
                    "",
                    &self.estilo("titulo"),
                );
                if gui.button(boton, "", &self.estilo("button_play")) {
                    self.play(self.hud.id, 0);
                    self.hud.estado = EstadoGui::InGameGui;
                    self.tiempo.escala = 1.0;
                }
            }
            EstadoGui::InGameGui => {
                let texto = self.hud.player_score.to_string();
                gui.label(
                    Rect::new(10.0, 100.0, 100.0, 40.0),
                    &texto,
                    &self.estilo("GUI"),
                );
            }
            EstadoGui::ShareGui => {
                // GUI.Window devuelve el mismo rectángulo, y Unity llama a ventanaFin después,
                // así que dentro de ella windowRectFin ya es este.
                let rect = Rect::new(ANCHO / 2.0 - 250.0, 100.0, 500.0, 400.0);
                self.hud.window_rect_fin = rect;
                let ventana = self.estilo("ventana");
                gui.ventana(0, rect, "", &ventana, |gui| self.ventana_fin(gui));
                if gui.button(boton, "", &self.estilo("button_play")) {
                    self.play(self.hud.id, 0);
                    self.estaticos.spawn_activo = true;
                    self.cargar = true;
                }
            }
        }
    }

    fn ventana_fin(&mut self, gui: &mut Gui) {
        let w = self.hud.window_rect_fin.w;
        gui.label(
            Rect::new(w / 2.0 - 256.0, 0.0, 512.0, 112.0),
            "Game Over",
            &self.estilo("base_titulo"),
        );
        if self.hud.player_score > self.hud.best_player_score {
            self.hud.new_record = true;
            self.hud.best_player_score = self.hud.player_score;
            self.preferencias
                .guardar_entero("BestScore", self.hud.player_score);
        }
        let puntaje = format!("Score: {}", self.hud.player_score);
        let fuente = self.estilo("font_default");
        if self.hud.new_record {
            gui.label(
                Rect::new(w / 2.0 - 175.0, 150.0, 350.0, 100.0),
                "New Record!",
                &self.estilo("label_record"),
            );
            gui.label(
                Rect::new(w / 2.0 - 225.0, 270.0, 450.0, 100.0),
                &puntaje,
                &fuente,
            );
        } else {
            gui.label(
                Rect::new(w / 2.0 - 225.0, 150.0, 450.0, 100.0),
                &puntaje,
                &fuente,
            );
            let mejor = format!("Best: {}", self.hud.best_player_score);
            gui.label(
                Rect::new(w / 2.0 - 200.0, 250.0, 400.0, 70.0),
                &mejor,
                &fuente,
            );
        }
    }

    // --- Spawn ------------------------------------------------------------------------------

    fn spawn_(&mut self) {
        let x = self.escena.posicion(self.spawn.id).x;
        let y = unity::azar_entre(self.spawn.y_min, self.spawn.y_max);
        let muro = self.spawn.muro.clone();
        self.instanciar(&muro, Some(vec3(x, y, 0.0)), Some(Quat::IDENTITY));
        if self.estaticos.spawn_activo {
            let espera = unity::azar_entre(self.spawn.spawn_min, self.spawn.spawn_max);
            self.spawn.invoke = Some(self.tiempo.tiempo + espera as f64);
        }
    }

    // --- PlatformerCharacter2D ----------------------------------------------------------------

    fn animator(&mut self) -> &mut unity::animator::Animator {
        self.escena
            .objeto_mut(self.personaje.id)
            .animator
            .as_mut()
            .expect("Animator")
    }

    fn personaje_fixed_update(&mut self) {
        let gc = self.escena.posicion(self.personaje.ground_check);
        let p = &self.personaje;
        self.personaje.grounded = self
            .escena
            .superpone_circulo(vec2(gc.x, gc.y), p.grounded_radius, p.what_is_ground)
            .is_some();
        let grounded = self.personaje.grounded;
        let particulas = self.escena.con_etiqueta("Particula");
        let muerto = self.animator().get_bool("Dead");
        if grounded & !muerto & particulas.is_empty() {
            let polvo = self.personaje.polvo.clone();
            let go = self.instanciar(&polvo, None, None);
            self.escena.fijar_posicion(go, vec3(gc.x - 1.0, gc.y, gc.z));
            self.capa_de_renderer(go, "Player");
            self.escena.fijar_padre(go, Some(self.personaje.id));
        }
        let compartir = self.estaticos.estado_static == EstadoGui::ShareGui;
        if !grounded && self.tiempo.escala > 0.0 {
            if compartir {
                self.stop(self.personaje.id, 1);
            } else {
                self.play(self.personaje.id, 1);
            }
        } else if compartir {
            self.stop(self.personaje.id, 1);
        }
        let vy = self.escena.velocidad(self.personaje.id).y;
        let anim = self.animator();
        anim.set_bool("Ground", grounded);
        anim.set_float("vSpeed", vy);
        if grounded && self.personaje.can_double_jump {
            self.personaje.double_jump = false;
        }
        let particulas = self.escena.con_etiqueta("Particula");
        if !grounded & !particulas.is_empty() {
            self.escena.fijar_padre(particulas[0], None);
        }
    }

    /// renderer.sortingLayerName = nombre.
    fn capa_de_renderer(&mut self, id: Id, nombre: &str) {
        let capa = self
            .proyecto
            .capas_orden
            .iter()
            .position(|c| c == nombre)
            .unwrap_or(0);
        let obj = self.escena.objeto_mut(id);
        if let Some(r) = obj.particulas.as_mut().and_then(|p| p.renderer.as_mut()) {
            r.capa_orden = capa;
        } else if let Some(s) = obj.sprite.as_mut() {
            s.capa_orden = capa;
        }
    }

    fn mover(&mut self, mut mover: f32, agachado: bool, saltar: bool) {
        let id = self.personaje.id;
        let grounded = self.personaje.grounded;
        if grounded || self.personaje.air_control {
            mover = if !agachado {
                mover
            } else {
                mover * self.personaje.crouch_speed
            };
            self.animator().set_float("Speed", mover.abs());
            let v = self.escena.velocidad(id);
            self.escena
                .fijar_velocidad(id, vec2(mover * self.personaje.max_speed, v.y));
            let derecha = self.personaje.facing_right;
            if (mover > 0.0 && !derecha) || (mover < 0.0 && derecha) {
                self.flip();
            }
        }
        let p = &self.personaje;
        if (grounded || (!p.double_jump & p.can_double_jump)) && saltar {
            self.animator().set_bool("Ground", false);
            let v = self.escena.velocidad(id);
            self.escena.fijar_velocidad(id, vec2(v.x, 0.0));
            if self.personaje.max_speed > 0.0 {
                self.escena
                    .agregar_fuerza(id, vec2(0.0, self.personaje.jump_force));
                self.play(id, 0);
                let gc = self.escena.posicion(self.personaje.ground_check);
                let hit = self.personaje.hit_salto.clone();
                let go = self.instanciar(&hit, None, None);
                self.escena
                    .fijar_posicion(go, vec3(gc.x + 1.0, gc.y + 1.0, gc.z));
                self.capa_de_renderer(go, "Player");
            }
            if !grounded & self.personaje.can_double_jump {
                self.personaje.double_jump = true;
            }
        }
    }

    fn flip(&mut self) {
        self.personaje.facing_right = !self.personaje.facing_right;
        self.escena.objeto_mut(self.personaje.id).local.escala.x *= -1.0;
    }

    // --- Triggers -----------------------------------------------------------------------------

    /// OnTriggerEnter2D de los scripts de `id`, con el colisionador de `otro`.
    fn on_trigger_enter(&mut self, id: Id, otro: Id) {
        if !self.escena.existe(id) || !self.escena.existe(otro) {
            return;
        }
        let scripts: Vec<String> = self
            .escena
            .objeto(id)
            .scripts
            .iter()
            .filter(|s| s.activo)
            .map(|s| s.nombre.clone())
            .collect();
        for s in scripts {
            match s.as_str() {
                "Muro" => self.muro_trigger(id, otro),
                "PowerUp" => self.power_up_trigger(id, otro),
                "Destroyer" => self.destroyer_trigger(otro),
                _ => {}
            }
        }
    }

    /// other.GetComponent<PlatformerCharacter2D>(): solo el personaje lo tiene.
    fn es_personaje(&self, otro: Id) -> bool {
        otro == self.personaje.id
    }

    fn muro_trigger(&mut self, id: Id, otro: Id) {
        if self.escena.objeto(otro).etiqueta == "Player"
            && self.es_personaje(otro)
            && self.personaje.max_speed > 0.0
        {
            self.estaticos.is_shake = true;
            self.play(id, 0);
            self.personaje.max_speed = 0.0;
            let anim = self.animator();
            anim.set_bool("Ground", true);
            anim.set_bool("Dead", true);
            self.estaticos.spawn_activo = false;
        }
    }

    fn power_up_trigger(&mut self, id: Id, otro: Id) {
        // `&` sin cortocircuito: con otro objeto, GetComponent devuelve null y la excepción
        // (NullReferenceException) corta el método sin efecto.
        if !self.es_personaje(otro) {
            return;
        }
        let activo = self.power_ups.get(&id).copied().unwrap_or(false);
        if (self.escena.objeto(otro).etiqueta == "Player")
            & activo
            & (self.personaje.max_speed > 0.0)
        {
            self.play(id, 0);
            self.increase_score(1);
            self.power_ups.insert(id, false);
            if let Some(s) = self.escena.objeto_mut(id).sprite.as_mut() {
                s.activo = false;
            }
        }
    }

    fn destroyer_trigger(&mut self, otro: Id) {
        let etiqueta = self.escena.objeto(otro).etiqueta.clone();
        if etiqueta == "Particulas" {
            self.escena.destruir(otro);
        }
        if etiqueta == "Suelo" {
            let mut p = self.escena.posicion(otro);
            p.x += self.amount;
            self.escena.fijar_posicion(otro, p);
        } else if let Some(padre) = self.escena.objeto(otro).padre {
            self.escena.destruir(padre);
        } else {
            self.escena.destruir(otro);
        }
    }

    // --- El frame -----------------------------------------------------------------------------

    /// Un frame de Unity que duró `real` segundos, hasta antes de dibujar.
    pub fn frame(&mut self, real: f32, entrada: Entrada) {
        if self.cargar {
            // Application.LoadLevel(0): la escena se vuelve a cargar entera; el tiempo y los
            // campos static siguen.
            self.cargar = false;
            self.escena = Escena::cargar(&self.proyecto);
            self.iniciar_escena();
        }
        self.tiempo.nuevo_frame(real);
        // FixedUpdate y Physics2D.
        while self.tiempo.siguiente_paso_fijo() {
            self.personaje_fixed_update();
            let saltar = self.control.jump;
            self.mover(1.0, false, saltar);
            self.control.jump = false;
            let contactos = self.escena.paso_fisica(self.tiempo.paso_fijo);
            for c in contactos {
                if c.entra && c.trigger {
                    self.on_trigger_enter(c.a, c.b);
                    self.on_trigger_enter(c.b, c.a);
                }
            }
        }
        // Update, en el orden de los objetos: Main Camera (CameraRunner, HUD) y Character.
        self.camera_runner_update();
        self.hud_update();
        if entrada.pulsado & (self.tiempo.escala > 0.0) {
            self.control.jump = true;
        }
        // Invoke y corrutinas.
        if let Some(t) = self.spawn.invoke
            && self.tiempo.tiempo >= t
        {
            self.spawn.invoke = None;
            self.spawn_();
        }
        self.corrutinas();
        // Animator y partículas.
        let dt = self.tiempo.delta;
        for id in self.escena.ids() {
            if !self.escena.activo_en_jerarquia(id) {
                continue;
            }
            let emisor = render::matriz_sin_escala(&self.escena, id);
            let obj = self.escena.objeto_mut(id);
            if let Some(a) = obj.animator.as_mut()
                && let Some(sprite) = a.actualizar(&self.proyecto.clips, dt)
                && let Some(s) = obj.sprite.as_mut()
            {
                s.sprite = Some(sprite);
            }
            if let Some(p) = obj.particulas.as_mut() {
                p.actualizar(dt, &emisor);
            }
        }
        self.escena.limpiar_destruidos();
    }

    /// CFX_AutoDestructShuriken.CheckIfAlive: cada 0,5 s mira si el sistema (con sus hijos)
    /// sigue vivo; si no, destruye el objeto.
    fn corrutinas(&mut self) {
        let ahora = self.tiempo.tiempo;
        let mut i = 0;
        while i < self.auto.len() {
            let a = &self.auto[i];
            if !self.escena.existe(a.id) || self.escena.objeto(a.id).destruido {
                self.auto.remove(i);
                continue;
            }
            if ahora < a.siguiente {
                i += 1;
                continue;
            }
            if self.vivo_con_hijos(a.id) {
                self.auto[i].siguiente += 0.5;
                i += 1;
                continue;
            }
            let a = self.auto.remove(i);
            if a.only_deactivate {
                self.escena.objeto_mut(a.id).activo = false;
            } else {
                self.escena.destruir(a.id);
            }
        }
    }

    fn vivo_con_hijos(&self, id: Id) -> bool {
        let o = self.escena.objeto(id);
        o.particulas.as_ref().is_some_and(|p| p.vivo())
            || o.hijos.iter().any(|&h| self.vivo_con_hijos(h))
    }

    /// El dibujo del frame y después OnGUI (los eventos del ratón y el Repaint).
    pub fn dibujar(&mut self, recursos: &Recursos, gui: &mut Gui, entrada: Entrada) {
        let camara = Camara::de(&self.escena, self.camara.id, ANCHO, ALTO);
        render::dibujar(&self.escena, &self.proyecto, recursos, &camara);
        gui.raton = entrada.raton;
        for evento in Gui::eventos(entrada.pulsado, entrada.soltado) {
            gui.empezar(evento);
            self.on_gui(gui);
        }
        gui.presentar();
    }
}
