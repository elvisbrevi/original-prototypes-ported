// Los scripts de Assembly-CSharp.dll y -firstpass traducidos del C# decompilado
// (reference/csharp), y el frame de Unity 5.1 que los llama: FixedUpdate con la animación en modo
// AnimatePhysics y la física 2D a paso fijo, la interpolación de los cuerpos, Update, el Animator
// en modo Normal, el dibujo, el GUILayer (el GUIText del contador de FPS) y OnGUI. Cada script es
// una estructura con sus campos; los campos static (PlatformerCharacter2D.m_Grounded y
// PlatformerDragon2D.m_Grounded) viven en `Estaticos`, fuera de la escena, y sobreviven a
// Application.LoadLevel.

use macroquad::prelude::*;
use unity::datos::{Estilo, Proyecto, Value};
use unity::escena::{Escena, Id};
use unity::fisica2d::Contacto2D;
use unity::gui::Gui;
use unity::matematica::{mover_hacia, signo, suavizar};
use unity::render::{self, Camara, Recursos};
use unity::tiempo::Tiempo;

/// Screen.width y Screen.height: el lienzo de la versión WebGL (960x600).
pub const ANCHO: f32 = 960.0;
pub const ALTO: f32 = 600.0;

/// Los radios de PlatformerCharacter2D (k_GroundedRadius y k_CeilingRadius).
const RADIO_SUELO: f32 = 0.2;
const RADIO_TECHO: f32 = 0.01;

/// Los campos static de los scripts.
#[derive(Default)]
struct Estaticos {
    /// PlatformerCharacter2D.m_Grounded.
    en_suelo: bool,
    /// PlatformerDragon2D.m_Grounded: se pone en false en cada FixedUpdate y nadie lo lee.
    dragon_en_suelo: bool,
}

/// PlatformerCharacter2D y Platformer2DUserControl, los dos scripts del héroe.
struct Heroe {
    id: Id,
    max_speed: f32,
    jump_force: f32,
    crouch_speed: f32,
    air_control: bool,
    what_is_ground: u32,
    ground_check: Id,
    ceiling_check: Id,
    facing_right: bool,
    /// Platformer2DUserControl.m_Jump.
    jump: bool,
    /// Platformer2DUserControl.h: la dirección en que corre.
    h: f32,
}

/// PlatformerDragon2D y Platformer2DDragonControl.
struct Dragon {
    id: Id,
    max_speed: f32,
    facing_right: bool,
    /// Platformer2DDragonControl.h.
    h: f32,
    /// Platformer2DDragonControl existe (Destroy(this) lo quita al final del frame).
    control: bool,
    destruir_control: bool,
}

struct Camera2DFollow {
    id: Id,
    target: Id,
    damping: f32,
    look_ahead_factor: f32,
    look_ahead_return_speed: f32,
    look_ahead_move_threshold: f32,
    offset_z: f32,
    last_target_position: Vec3,
    current_velocity: Vec3,
    look_ahead_pos: Vec3,
}

struct GuiPrototipo {
    pause_game: bool,
    show_gui: bool,
    cantidad_niveles: i32,
    ancho_boton: f32,
    window_rect: Rect,
}

/// FPSCounter (Standard Assets), con su GUIText.
struct ContadorFps {
    id: Id,
    acumulador: i32,
    siguiente: f64,
    actual: i32,
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
    /// Input.GetKeyDown("p").
    pub tecla_p: bool,
    /// Input.GetKey(KeyCode.LeftControl).
    pub control: bool,
}

pub struct Juego {
    proyecto: Proyecto,
    pub escena: Escena,
    pub tiempo: Tiempo,
    /// Application.loadedLevel.
    pub nivel: usize,
    /// Time.realtimeSinceStartup.
    reloj: f64,
    estaticos: Estaticos,
    heroe: Heroe,
    dragones: Vec<Dragon>,
    camara: Camera2DFollow,
    menu: GuiPrototipo,
    fps: Option<ContadorFps>,
    /// Application.LoadLevel pendiente: se carga al empezar el frame siguiente.
    cargar: Option<usize>,
}

fn campo_f32(campos: &Value, nombre: &str) -> f32 {
    campos[nombre].as_f64().unwrap_or(0.0) as f32
}

impl Juego {
    pub fn nuevo(proyecto: Proyecto) -> Self {
        let escena = Escena::cargar_nivel(&proyecto, 0);
        let tiempo = Tiempo::nuevo(&proyecto.tiempo);
        let mut juego = Self {
            heroe: Heroe {
                id: 0,
                max_speed: 0.0,
                jump_force: 0.0,
                crouch_speed: 0.0,
                air_control: false,
                what_is_ground: 0,
                ground_check: 0,
                ceiling_check: 0,
                facing_right: true,
                jump: false,
                h: 1.0,
            },
            dragones: Vec::new(),
            camara: Camera2DFollow {
                id: 0,
                target: 0,
                damping: 0.0,
                look_ahead_factor: 0.0,
                look_ahead_return_speed: 0.0,
                look_ahead_move_threshold: 0.0,
                offset_z: 0.0,
                last_target_position: Vec3::ZERO,
                current_velocity: Vec3::ZERO,
                look_ahead_pos: Vec3::ZERO,
            },
            menu: GuiPrototipo {
                pause_game: false,
                show_gui: false,
                cantidad_niveles: 4,
                ancho_boton: 100.0,
                window_rect: Rect::default(),
            },
            fps: None,
            cargar: None,
            nivel: 0,
            reloj: 0.0,
            estaticos: Estaticos::default(),
            proyecto,
            escena,
            tiempo,
        };
        juego.iniciar_escena();
        juego
    }

    fn estilo(&self, nombre: &str) -> Estilo {
        self.proyecto.estilos[nombre].clone()
    }

    fn con_script(&self, script: &str) -> Vec<(Id, Value)> {
        self.escena
            .ids()
            .into_iter()
            .filter_map(|id| {
                let s = self.escena.objeto(id).script(script)?;
                Some((id, s.campos.clone()))
            })
            .collect()
    }

    fn hijo(&self, id: Id, nombre: &str) -> Id {
        self.escena
            .objeto(id)
            .hijos
            .iter()
            .copied()
            .find(|&h| self.escena.objeto(h).nombre == nombre)
            .expect("hijo")
    }

    /// Los Awake y los Start de la escena recién cargada.
    fn iniciar_escena(&mut self) {
        // PlatformerCharacter2D.Awake: transform.Find("GroundCheck") y "CeilingCheck".
        let (id, c) = self.con_script("PlatformerCharacter2D").remove(0);
        self.heroe = Heroe {
            id,
            max_speed: campo_f32(&c, "m_MaxSpeed"),
            jump_force: campo_f32(&c, "m_JumpForce"),
            crouch_speed: campo_f32(&c, "m_CrouchSpeed"),
            air_control: c["m_AirControl"].as_i64() == Some(1),
            what_is_ground: c["m_WhatIsGround"].as_u64().unwrap_or(0) as u32,
            ground_check: self.hijo(id, "GroundCheck"),
            ceiling_check: self.hijo(id, "CeilingCheck"),
            facing_right: true,
            jump: false,
            h: 1.0,
        };
        self.dragones = self
            .con_script("PlatformerDragon2D")
            .into_iter()
            .map(|(id, c)| Dragon {
                id,
                max_speed: campo_f32(&c, "m_MaxSpeed"),
                facing_right: false,
                h: 1.0,
                control: self
                    .escena
                    .objeto(id)
                    .script("Platformer2DDragonControl")
                    .is_some(),
                destruir_control: false,
            })
            .collect();
        let (id, c) = self.con_script("Camera2DFollow").remove(0);
        self.camara = Camera2DFollow {
            id,
            target: self
                .escena
                .original(c["target"]["objeto"].as_i64().expect("target")),
            damping: campo_f32(&c, "damping"),
            look_ahead_factor: campo_f32(&c, "lookAheadFactor"),
            look_ahead_return_speed: campo_f32(&c, "lookAheadReturnSpeed"),
            look_ahead_move_threshold: campo_f32(&c, "lookAheadMoveThreshold"),
            offset_z: 0.0,
            last_target_position: Vec3::ZERO,
            current_velocity: Vec3::ZERO,
            look_ahead_pos: Vec3::ZERO,
        };
        let (_, c) = self.con_script("GUI_Prototipo").remove(0);
        let r = &c["windowRect"];
        self.menu = GuiPrototipo {
            pause_game: c["pauseGame"].as_i64() == Some(1),
            show_gui: false,
            cantidad_niveles: 4,
            ancho_boton: 100.0,
            window_rect: Rect::new(
                r[0].as_f64().unwrap_or(0.0) as f32,
                r[1].as_f64().unwrap_or(0.0) as f32,
                r[2].as_f64().unwrap_or(0.0) as f32,
                r[3].as_f64().unwrap_or(0.0) as f32,
            ),
        };
        self.fps = self
            .con_script("FPSCounter")
            .into_iter()
            .next()
            .map(|(id, _)| ContadorFps {
                id,
                acumulador: 0,
                siguiente: 0.0,
                actual: 0,
            });
        self.escena.sincronizar_fisica();
        // Start: Camera2DFollow y FPSCounter.
        self.camera_follow_start();
        if let Some(f) = &mut self.fps {
            f.siguiente = self.reloj + 0.5;
        }
    }

    // --- Camera2DFollow ---------------------------------------------------------------------

    fn camera_follow_start(&mut self) {
        let t = self.escena.posicion(self.camara.target);
        self.camara.last_target_position = t;
        self.camara.offset_z = (self.escena.posicion(self.camara.id) - t).z;
        self.escena.fijar_padre(self.camara.id, None);
    }

    fn camera_follow_update(&mut self) {
        let dt = self.tiempo.delta;
        let c = &mut self.camara;
        let target = self.escena.posicion(c.target);
        let x = (target - c.last_target_position).x;
        if x.abs() > c.look_ahead_move_threshold {
            c.look_ahead_pos = c.look_ahead_factor * Vec3::X * signo(x);
        } else {
            c.look_ahead_pos =
                mover_hacia(c.look_ahead_pos, Vec3::ZERO, dt * c.look_ahead_return_speed);
        }
        // Vector3.forward es (0, 0, 1).
        let v = target + c.look_ahead_pos + Vec3::Z * c.offset_z;
        let num = 1.0;
        let actual = self.escena.posicion(c.id);
        let p = suavizar(
            actual,
            vec3(v.x, v.y + num, v.z),
            &mut c.current_velocity,
            c.damping,
            f32::INFINITY,
            dt,
        );
        self.escena.fijar_posicion(c.id, p);
        self.camara.last_target_position = target;
    }

    // --- GUI_Prototipo ----------------------------------------------------------------------

    fn gui_prototipo_update(&mut self, entrada: Entrada) {
        let m = &mut self.menu;
        if entrada.tecla_p {
            m.pause_game = !m.pause_game;
        }
        if m.pause_game {
            self.tiempo.escala = 0.0;
            m.pause_game = true;
            m.show_gui = true;
        }
        if !m.pause_game {
            self.tiempo.escala = 1.0;
            m.pause_game = false;
            m.show_gui = false;
        }
    }

    /// GUI_Prototipo.OnGUI, una vez por evento.
    fn on_gui(&mut self, gui: &mut Gui) {
        if !self.menu.show_gui {
            return;
        }
        let (cantidad, ancho) = (self.menu.cantidad_niveles, self.menu.ancho_boton);
        let num = ancho * (cantidad / 2) as f32 + 20.0;
        let num2 = ancho * 2.0 + 40.0;
        // Screen.width / 2 es una división entera.
        let rect = Rect::new(
            ((ANCHO as i32) / 2) as f32 - num / 2.0,
            ((ALTO as i32) / 2) as f32 - num2 / 2.0,
            num,
            num2,
        );
        self.menu.window_rect = rect;
        let ventana = self.estilo("window");
        let boton = self.estilo("button");
        let mut elegido = None;
        self.menu.window_rect = gui.ventana(0, rect, "Nivel", &ventana, |gui| {
            // CrearWindow.
            let mut num = 1;
            for i in 0..2 {
                for j in 0..cantidad / 2 {
                    let r = Rect::new(
                        10.0 + j as f32 * ancho,
                        30.0 + i as f32 * ancho,
                        ancho,
                        ancho,
                    );
                    if gui.button(r, &num.to_string(), &boton) {
                        elegido = Some(num - 1);
                    }
                    num += 1;
                }
            }
        });
        if let Some(n) = elegido {
            self.load_level(n);
        }
    }

    /// Application.LoadLevel(n): si la escena no está en el build, Unity solo escribe un error.
    fn load_level(&mut self, n: i32) {
        if n >= 0 && (n as usize) < self.proyecto.niveles() {
            self.cargar = Some(n as usize);
        }
    }

    // --- PlatformerCharacter2D y Platformer2DUserControl --------------------------------------

    fn animator_heroe(&mut self) -> &mut unity::animator::Animator {
        self.escena
            .objeto_mut(self.heroe.id)
            .animator
            .as_mut()
            .expect("Animator")
    }

    fn personaje_fixed_update(&mut self) {
        self.estaticos.en_suelo = false;
        let gc = self.escena.posicion(self.heroe.ground_check);
        let tocados = self.escena.superpone_circulo_todos(
            vec2(gc.x, gc.y),
            RADIO_SUELO,
            self.heroe.what_is_ground,
        );
        for (objeto, _) in tocados {
            if objeto != self.heroe.id {
                self.estaticos.en_suelo = true;
            }
        }
        let en_suelo = self.estaticos.en_suelo;
        let vy = self.escena.velocidad(self.heroe.id).y;
        let anim = self.animator_heroe();
        anim.set_bool("Ground", en_suelo);
        anim.set_float("vSpeed", vy);
    }

    fn mover(&mut self, mut mover: f32, mut agachado: bool, saltar: bool) {
        let id = self.heroe.id;
        if !agachado && self.animator_heroe().get_bool("Crouch") {
            let cc = self.escena.posicion(self.heroe.ceiling_check);
            if self
                .escena
                .superpone_circulo(vec2(cc.x, cc.y), RADIO_TECHO, self.heroe.what_is_ground)
                .is_some()
            {
                agachado = true;
            }
        }
        self.animator_heroe().set_bool("Crouch", agachado);
        if self.estaticos.en_suelo || self.heroe.air_control {
            mover = if !agachado {
                mover
            } else {
                mover * self.heroe.crouch_speed
            };
            self.animator_heroe().set_float("Speed", mover.abs());
            let v = self.escena.velocidad(id);
            self.escena
                .fijar_velocidad(id, vec2(mover * self.heroe.max_speed, v.y));
            let derecha = self.heroe.facing_right;
            if (mover > 0.0 && !derecha) || (mover < 0.0 && derecha) {
                self.flip(id);
            }
        }
        if self.estaticos.en_suelo && saltar && self.animator_heroe().get_bool("Ground") {
            self.estaticos.en_suelo = false;
            self.animator_heroe().set_bool("Ground", false);
            self.escena
                .agregar_fuerza(id, vec2(0.0, self.heroe.jump_force));
        }
    }

    /// Flip de PlatformerCharacter2D y de PlatformerDragon2D.
    fn flip(&mut self, id: Id) {
        if id == self.heroe.id {
            self.heroe.facing_right = !self.heroe.facing_right;
        } else if let Some(d) = self.dragones.iter_mut().find(|d| d.id == id) {
            d.facing_right = !d.facing_right;
        }
        self.escena.objeto_mut(id).local.escala.x *= -1.0;
    }

    fn user_control_fixed_update(&mut self, entrada: Entrada) {
        let tecla = entrada.control;
        let (h, saltar) = (self.heroe.h, self.heroe.jump);
        self.mover(h, tecla, saltar);
        self.heroe.jump = false;
    }

    /// Platformer2DUserControl.muere.
    fn muere(&mut self) {
        self.cargar = Some(self.nivel);
    }

    fn user_control_trigger(&mut self, otro: Id) {
        let nombre = self.escena.objeto(otro).nombre.clone();
        if nombre.contains("Final") {
            // Debug.Log("Fin") y GameObject.Find("Main Camera").GetComponent<GUI_Prototipo>().
            self.menu.pause_game = true;
        }
        if nombre.contains("Spike") {
            self.muere();
        }
    }

    fn user_control_collision(&mut self, otro: Id, colisionador: usize) {
        let nombre = self.escena.objeto(otro).nombre.clone();
        if nombre.contains("ParedRight") {
            self.heroe.h = -1.0;
        } else if nombre.contains("ParedLeft") {
            self.heroe.h = 1.0;
        }
        if nombre.contains("Dragon") {
            // collider.bounds.center.
            let Some((lo, hi)) = self.escena.limites(otro, colisionador) else {
                return;
            };
            let centro = (lo + hi) / 2.0;
            if self.escena.posicion(self.heroe.ground_check).y > centro.y {
                self.escena.destruir(otro);
            } else {
                self.muere();
            }
        }
    }

    // --- PlatformerDragon2D y Platformer2DDragonControl ---------------------------------------

    fn dragon_fixed_update(&mut self, i: usize) {
        // PlatformerDragon2D.FixedUpdate.
        self.estaticos.dragon_en_suelo = false;
        // Platformer2DDragonControl.FixedUpdate: Move(h, false, false).
        let d = &self.dragones[i];
        if !d.control {
            return;
        }
        let (id, mover, max) = (d.id, d.h, d.max_speed);
        let v = self.escena.velocidad(id);
        self.escena.fijar_velocidad(id, vec2(mover * max, v.y));
        let derecha = self.dragones[i].facing_right;
        if (mover > 0.0 && !derecha) || (mover < 0.0 && derecha) {
            self.flip(id);
        }
    }

    fn dragon_trigger(&mut self, i: usize, otro: Id) {
        if !self.dragones[i].control {
            return;
        }
        if self.escena.objeto(otro).nombre.contains("Spike") {
            let d = &mut self.dragones[i];
            d.h = 1.0;
            // Object.Destroy(this): el script se va al final del frame.
            d.destruir_control = true;
        }
    }

    fn dragon_collision(&mut self, i: usize, otro: Id) {
        if !self.dragones[i].control {
            return;
        }
        let nombre = &self.escena.objeto(otro).nombre;
        if nombre.contains("ParedRight") {
            self.dragones[i].h = -1.0;
        } else if nombre.contains("ParedLeft") {
            self.dragones[i].h = 1.0;
        }
    }

    // --- Mensajes de la física ------------------------------------------------------------------

    /// OnTriggerEnter2D u OnCollisionEnter2D de los scripts de `id` con el colisionador `ci` de
    /// `otro`.
    fn mensaje(&mut self, id: Id, otro: Id, ci: usize, trigger: bool) {
        if !self.escena.existe(id) || !self.escena.existe(otro) {
            return;
        }
        if id == self.heroe.id {
            if trigger {
                self.user_control_trigger(otro);
            } else {
                self.user_control_collision(otro, ci);
            }
        } else if let Some(i) = self.dragones.iter().position(|d| d.id == id) {
            if trigger {
                self.dragon_trigger(i, otro);
            } else {
                self.dragon_collision(i, otro);
            }
        }
    }

    fn al_tocar(&mut self, c: Contacto2D) {
        if c.entra {
            self.mensaje(c.a, c.b, c.cb, c.trigger);
            self.mensaje(c.b, c.a, c.ca, c.trigger);
        }
    }

    // --- FPSCounter -----------------------------------------------------------------------------

    fn fps_update(&mut self) {
        let reloj = self.reloj;
        let Some(f) = &mut self.fps else {
            return;
        };
        f.acumulador += 1;
        if reloj > f.siguiente {
            f.actual = (f.acumulador as f32 / 0.5) as i32;
            f.acumulador = 0;
            f.siguiente += 0.5;
            let texto = format!("{} FPS", f.actual);
            if let Some(t) = self.escena.objeto_mut(f.id).texto_gui.as_mut() {
                t.texto = texto;
            }
        }
    }

    // --- El frame -----------------------------------------------------------------------------

    /// El Animator de los objetos en un modo (AnimatePhysics en el paso fijo, Normal en Update):
    /// el sprite y las propiedades que fija el clip.
    fn animar(&mut self, fisica: bool, dt: f32) {
        for id in self.escena.ids() {
            if !self.escena.activo_en_jerarquia(id) {
                continue;
            }
            let obj = self.escena.objeto_mut(id);
            let Some(a) = obj.animator.as_mut() else {
                continue;
            };
            if (a.modo == 1) != fisica {
                continue;
            }
            let sprite = a.actualizar(&self.proyecto.clips, dt);
            let valores = a.valores(&self.proyecto.clips);
            let con_defectos = !a.defectos.is_empty();
            if let (Some(sprite), Some(s)) = (sprite, obj.sprite.as_mut()) {
                s.sprite = Some(sprite);
            }
            if !valores.is_empty() || con_defectos {
                self.escena.animar_colisionador(id, valores);
            }
        }
    }

    /// Un frame de Unity que duró `real` segundos, hasta antes de dibujar.
    pub fn frame(&mut self, real: f32, entrada: Entrada) {
        if let Some(n) = self.cargar.take() {
            // Application.LoadLevel: la escena se vuelve a cargar entera; el tiempo y los campos
            // static siguen.
            self.nivel = n;
            self.escena = Escena::cargar_nivel(&self.proyecto, n);
            self.iniciar_escena();
        }
        self.reloj += real as f64;
        self.tiempo.nuevo_frame(real);
        // FixedUpdate, la animación de los Animator en AnimatePhysics y Physics2D.
        let paso = self.tiempo.paso_fijo as f64;
        if self.tiempo.tiempo_fijo + paso <= self.tiempo.tiempo {
            self.escena.restaurar_interpolados();
        }
        while self.tiempo.siguiente_paso_fijo() {
            let mut orden: Vec<Id> = std::iter::once(self.heroe.id)
                .chain(self.dragones.iter().map(|d| d.id))
                .collect();
            orden.sort();
            for id in orden {
                if id == self.heroe.id {
                    self.personaje_fixed_update();
                    self.user_control_fixed_update(entrada);
                } else if let Some(i) = self.dragones.iter().position(|d| d.id == id) {
                    self.dragon_fixed_update(i);
                }
            }
            self.animar(true, self.tiempo.paso_fijo);
            let contactos = self.escena.paso_fisica(self.tiempo.paso_fijo);
            for c in contactos {
                self.al_tocar(c);
            }
        }
        let t = (self.tiempo.tiempo - self.tiempo.tiempo_fijo) / paso;
        self.escena.interpolar(t as f32);
        // Update, en el orden de los objetos.
        let mut orden = vec![(self.camara.id, 0), (self.heroe.id, 1)];
        if let Some(f) = &self.fps {
            orden.push((f.id, 2));
        }
        orden.sort();
        for (_, script) in orden {
            match script {
                0 => {
                    self.camera_follow_update();
                    self.gui_prototipo_update(entrada);
                }
                1 => {
                    if !self.heroe.jump && entrada.pulsado {
                        self.heroe.jump = true;
                    }
                }
                _ => self.fps_update(),
            }
        }
        // El Animator en modo Normal.
        self.animar(false, self.tiempo.delta);
        // Lo destruido en el frame.
        for d in &mut self.dragones {
            if d.destruir_control {
                d.destruir_control = false;
                d.control = false;
            }
        }
        self.escena.limpiar_destruidos();
        let escena = &self.escena;
        self.dragones.retain(|d| escena.existe(d.id));
    }

    /// El dibujo del frame, el GUILayer y después OnGUI (los eventos del ratón y el Repaint).
    pub fn dibujar(&mut self, recursos: &Recursos, gui: &mut Gui, entrada: Entrada) {
        let camara = Camara::de(&self.escena, self.camara.id, ANCHO, ALTO);
        render::dibujar(&self.escena, &self.proyecto, recursos, &camara);
        for id in self.escena.ids() {
            if !self.escena.activo_en_jerarquia(id) {
                continue;
            }
            if let Some(t) = &self.escena.objeto(id).texto_gui {
                let p = self.escena.posicion(id);
                gui.texto_gui(t, vec2(p.x, p.y), ANCHO, ALTO);
            }
        }
        gui.raton = entrada.raton;
        for evento in Gui::eventos(entrada.pulsado, entrada.soltado) {
            gui.empezar(evento);
            self.on_gui(gui);
        }
        gui.presentar();
    }
}
