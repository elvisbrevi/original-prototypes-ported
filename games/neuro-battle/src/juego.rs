// Los scripts de Neuro Battle reconstruidos del asm.js (reference/csharp y reference/unityscript)
// y el frame de Unity 5.1 que los llama: Application.LoadLevel al empezar el frame siguiente, la
// física 2D a paso fijo con los OnTriggerEnter2D/OnCollisionEnter2D, los Start pendientes, los
// Update en el orden de los objetos, el Animator, el dibujo y OnGUI. Lo destruido se borra después
// de la física y después de los Update.
//
// Cada script es una variante de `Script` con sus campos; los static (Aparicion.contadorTiempo y
// BaseRotacion.choque) viven en `Juego` y sobreviven a LoadLevel. Un script que lee un objeto
// destruido corta su método ahí, como el NullReferenceException del original.

use macroquad::prelude::*;
use unity::audio::Audio;
use unity::azar_entero;
use unity::datos::{DatosObjeto, Estilo, Proyecto, Value};
use unity::entrada::{Entrada, Teclas};
use unity::escena::{Escena, Id};
use unity::fisica2d::Contacto2D;
use unity::gui::Gui;
use unity::matematica::{mover_hacia, signo, suavizar};
use unity::render::{self, Camara, Recursos};
use unity::tiempo::Tiempo;

/// Screen.width y Screen.height: el lienzo de la versión WebGL (960x600).
pub const ANCHO: f32 = 960.0;
pub const ALTO: f32 = 600.0;

/// Las escenas de BuildSettings.
const NIVEL: usize = 2;
const GAME_OVER: usize = 3;

/// La entrada del frame: las teclas del InputManager (con sus nombres de Unity) y las que el
/// código lee con Input.GetKey.
pub struct EntradaFrame<'a> {
    pub teclas: Teclas<'a>,
    /// Input.GetKey(KeyCode.E) y GetKeyUp.
    pub e: bool,
    pub e_soltada: bool,
    /// Input.GetKey(KeyCode.Escape).
    pub escape: bool,
    /// Input.GetKey(KeyCode.RightShift) y GetKeyUp.
    pub disparo: bool,
    pub disparo_soltado: bool,
}

struct Aparicion {
    timer_max: f32,
    timer: f32,
    /// Los prefabs de las neuronas (None si el campo no tenía uno).
    neuronas: Vec<Option<String>>,
    zoom: bool,
}

struct BaseRotacion {
    velocidad: f32,
    up_button: bool,
}

struct Union {
    /// Lo escribe BaseRotacion y nadie lo lee.
    estado: String,
    sonido: Option<String>,
}

struct Disparar {
    bala: Option<String>,
    desplazamiento: Vec3,
    disparo: bool,
    valido: bool,
    sonido: Option<String>,
}

#[derive(Clone)]
struct IaEnemy {
    target: Option<Id>,
    waypoint: Option<Id>,
    velocidad_x: f32,
    velocidad_y: f32,
    // velocidadHuida no lo lee nadie, ni siquiera el estado «huir».
    /// EstadoEnemigo: 0 quieto, 1 huir, 2 atacar, 3 patrullar.
    estado: i32,
    distancia_target: f32,
    distancia_waypoint: f32,
    distancia_muerte: f32,
    tiempo_quieto: i32,
    /// Privado (no se serializa): un clon empieza en 0.
    tiempo_quieto_check: i32,
}

struct Generador {
    /// El objeto que clona (el enemigo de la escena) y su id en el archivo.
    objetos: Vec<Option<(Id, i64)>>,
    tiempo_lanzar: i32,
    tiempo_lanzar_check: i32,
}

struct Camera2DFollow {
    target: Option<Id>,
    damping: f32,
    look_ahead_factor: f32,
    look_ahead_return_speed: f32,
    look_ahead_move_threshold: f32,
    offset_z: f32,
    last_target_position: Vec3,
    current_velocity: Vec3,
    look_ahead_pos: Vec3,
}

enum Script {
    Aparicion(Aparicion),
    BaseMovimiento { velocidad: f32 },
    BaseRotacion(BaseRotacion),
    Camera2DFollow(Camera2DFollow),
    Union(Union),
    AnimacionUnion,
    Bala,
    Bordescript,
    Disparar(Disparar),
    MuerteEnemigo { vida: f32 },
    IaEnemy(IaEnemy),
    Generador(Generador),
    Titulo { textura: String },
    Intrucciones { textura: String },
    GameOver { textura: String, estilo: String },
}

/// Un MonoBehaviour vivo: su objeto, su script y si ya pasó por Start.
struct Comportamiento {
    id: Id,
    script: Script,
    iniciado: bool,
}

pub struct Juego {
    proyecto: Proyecto,
    pub escena: Escena,
    pub tiempo: Tiempo,
    /// Application.loadedLevel.
    pub nivel: usize,
    entrada: Entrada,
    audio: Audio,
    comportamientos: Vec<Option<Comportamiento>>,
    /// Aparicion.contadorTiempo: nunca vuelve a 0.
    contador_tiempo: f32,
    /// BaseRotacion.choque: lo escriben los Union y nadie lo lee.
    choque: bool,
    /// PlayerPrefs "tiempo".
    tiempo_guardado: f32,
    /// Application.LoadLevel pendiente: se carga al empezar el frame siguiente.
    cargar: Option<usize>,
}

fn numero(v: &Value) -> f32 {
    v.as_f64().unwrap_or(0.0) as f32
}

fn vector(v: &Value) -> Vec3 {
    vec3(numero(&v[0]), numero(&v[1]), numero(&v[2]))
}

fn prefab(v: &Value) -> Option<String> {
    v["prefab"].as_str().map(str::to_owned)
}

/// El nombre del recurso de una referencia ({"AudioClip": nombre}).
fn recurso(v: &Value, tipo: &str) -> Option<String> {
    v[tipo].as_str().map(str::to_owned)
}

/// Transform.Translate(v) en el espacio propio: la posición avanza v girado por la rotación (sin
/// la escala).
fn trasladar(escena: &mut Escena, id: Id, v: Vec3) {
    let p = escena.posicion(id) + escena.rotacion(id) * v;
    escena.fijar_posicion(id, p);
}

impl Juego {
    pub fn nuevo(proyecto: Proyecto, audio: Audio) -> Self {
        let tiempo = Tiempo::nuevo(&proyecto.tiempo);
        let entrada = Entrada::nueva(&proyecto.ejes);
        let escena = Escena::cargar_nivel(&proyecto, 0);
        let mut juego = Self {
            proyecto,
            escena,
            tiempo,
            nivel: 0,
            entrada,
            audio,
            comportamientos: Vec::new(),
            contador_tiempo: 0.0,
            choque: false,
            tiempo_guardado: 0.0,
            cargar: None,
        };
        juego.crear_comportamientos(&juego.escena.ids());
        juego
    }

    /// Application.LoadLevel(n): la escena se carga al empezar el frame siguiente.
    fn load_level(&mut self, n: usize) {
        if n < self.proyecto.niveles() {
            self.cargar = Some(n);
        }
    }

    // --- MonoBehaviour --------------------------------------------------------------------------

    /// Awake: los scripts de los objetos recién creados, con sus campos serializados.
    fn crear_comportamientos(&mut self, ids: &[Id]) {
        for &id in ids {
            let scripts: Vec<(String, Value)> = self
                .escena
                .objeto(id)
                .scripts
                .iter()
                .filter(|s| s.activo)
                .map(|s| (s.nombre.clone(), s.campos.clone()))
                .collect();
            for (nombre, c) in scripts {
                let objeto =
                    |s: &Self, v: &Value| v["objeto"].as_i64().map(|o| s.escena.original(o));
                let script = match nombre.as_str() {
                    "Aparicion" => Script::Aparicion(Aparicion {
                        timer_max: numero(&c["timerMax"]),
                        timer: 0.0,
                        neuronas: c["neuronas"]
                            .as_array()
                            .map(|a| a.iter().map(prefab).collect())
                            .unwrap_or_default(),
                        zoom: false,
                    }),
                    "BaseMovimiento" => Script::BaseMovimiento {
                        velocidad: numero(&c["velocidadMovimiento"]),
                    },
                    "BaseRotacion" => Script::BaseRotacion(BaseRotacion {
                        velocidad: numero(&c["velocidadRotacion"]),
                        up_button: true,
                    }),
                    "Camera2DFollow" => Script::Camera2DFollow(Camera2DFollow {
                        target: objeto(self, &c["target"]),
                        damping: numero(&c["damping"]),
                        look_ahead_factor: numero(&c["lookAheadFactor"]),
                        look_ahead_return_speed: numero(&c["lookAheadReturnSpeed"]),
                        look_ahead_move_threshold: numero(&c["lookAheadMoveThreshold"]),
                        offset_z: 0.0,
                        last_target_position: Vec3::ZERO,
                        current_velocity: Vec3::ZERO,
                        look_ahead_pos: Vec3::ZERO,
                    }),
                    "Union" => Script::Union(Union {
                        estado: c["estado"].as_str().unwrap_or("libre").to_owned(),
                        sonido: recurso(&c["shootSound"], "AudioClip"),
                    }),
                    "animacionUnion" => Script::AnimacionUnion,
                    "bala" => Script::Bala,
                    "bordescript" => Script::Bordescript,
                    "disparar" => Script::Disparar(Disparar {
                        bala: prefab(&c["bala"]),
                        desplazamiento: vector(&c["bulletOffset"]),
                        disparo: false,
                        valido: c["valido"].as_i64().unwrap_or(0) != 0,
                        sonido: recurso(&c["shootSound"], "AudioClip"),
                    }),
                    "muerte_enemigo" => Script::MuerteEnemigo {
                        vida: numero(&c["vida"]),
                    },
                    "IA_Enemy" => Script::IaEnemy(IaEnemy {
                        target: objeto(self, &c["target"]),
                        waypoint: objeto(self, &c["waypoint"]),
                        velocidad_x: numero(&c["velocidadMovimiento_x"]),
                        velocidad_y: numero(&c["velocidadMovimiento_y"]),
                        estado: c["estadoEnemigo"].as_i64().unwrap_or(0) as i32,
                        distancia_target: numero(&c["distanciaTarget"]),
                        distancia_waypoint: numero(&c["distanciaWaypoint"]),
                        distancia_muerte: numero(&c["distanciaMuerte"]),
                        tiempo_quieto: c["tiempoQuieto"].as_i64().unwrap_or(0) as i32,
                        tiempo_quieto_check: 0,
                    }),
                    "GeneradorVitaminas" => Script::Generador(Generador {
                        objetos: c["objetos"]
                            .as_array()
                            .map(|a| {
                                a.iter()
                                    .map(|o| {
                                        let f = o["objeto"].as_i64()?;
                                        Some((self.escena.original(f), f))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        tiempo_lanzar: c["tiempoLanzar"].as_i64().unwrap_or(0) as i32,
                        tiempo_lanzar_check: 0,
                    }),
                    "titulo" => Script::Titulo {
                        textura: recurso(&c["aTexture"], "Texture2D").unwrap_or_default(),
                    },
                    "intrucciones" => Script::Intrucciones {
                        textura: recurso(&c["aTexture"], "Texture2D").unwrap_or_default(),
                    },
                    "gameover" => Script::GameOver {
                        textura: recurso(&c["aTexture"], "Texture2D").unwrap_or_default(),
                        estilo: c["estilo"]["estilo"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned(),
                    },
                    _ => continue,
                };
                self.comportamientos.push(Some(Comportamiento {
                    id,
                    script,
                    iniciado: false,
                }));
            }
        }
    }

    /// Los objetos de una jerarquía, en el orden en que se crearon.
    fn jerarquia(&self, raiz: Id) -> Vec<Id> {
        let mut ids = vec![raiz];
        let mut i = 0;
        while i < ids.len() {
            ids.extend(self.escena.objeto(ids[i]).hijos.iter().copied());
            i += 1;
        }
        ids.sort();
        ids
    }

    /// Object.Instantiate de un prefab.
    fn instanciar(&mut self, prefab: &str, posicion: Vec3, rotacion: Quat) -> Option<Id> {
        let datos = self.proyecto.prefabs.get(prefab)?.clone();
        let raiz = self
            .escena
            .instanciar_datos(&self.proyecto, &datos, posicion, rotacion);
        let ids = self.jerarquia(raiz);
        self.crear_comportamientos(&ids);
        Some(raiz)
    }

    /// Object.Instantiate de un objeto de la escena (el enemigo que clonan los generadores): la
    /// copia lleva el estado serializado de sus scripts en ese momento (los campos públicos de
    /// IA_Enemy, la vida de muerte_enemigo) y empieza su animación de nuevo.
    fn clonar(&mut self, (fuente, archivo): (Id, i64), posicion: Vec3, rotacion: Quat) {
        let Some(datos) = self.proyecto.escenas[self.nivel]
            .iter()
            .find(|d| d.id == archivo)
            .cloned()
        else {
            return;
        };
        let raiz = self
            .escena
            .instanciar_datos(&self.proyecto, &[datos], posicion, rotacion);
        let escala = self.escena.objeto(fuente).local.escala;
        self.escena.objeto_mut(raiz).local.escala = escala;
        // Los campos serializados de los scripts del original, en este momento.
        let ia = self
            .comportamientos
            .iter()
            .flatten()
            .find_map(|c| match &c.script {
                Script::IaEnemy(ia) if c.id == fuente => Some(ia.clone()),
                _ => None,
            });
        let vida = self
            .comportamientos
            .iter()
            .flatten()
            .find_map(|c| match c.script {
                Script::MuerteEnemigo { vida } if c.id == fuente => Some(vida),
                _ => None,
            });
        let primero = self.comportamientos.len();
        self.crear_comportamientos(&[raiz]);
        for c in self.comportamientos[primero..].iter_mut().flatten() {
            match &mut c.script {
                Script::IaEnemy(s) => {
                    if let Some(original) = &ia {
                        *s = IaEnemy {
                            tiempo_quieto_check: 0,
                            ..original.clone()
                        };
                    }
                }
                Script::MuerteEnemigo { vida: v } => {
                    if let Some(original) = vida {
                        *v = original;
                    }
                }
                _ => {}
            }
        }
    }

    /// Si un objeto sigue en la escena (Unity compara con null a los destruidos).
    fn vivo(&self, id: Option<Id>) -> Option<Id> {
        id.filter(|&i| self.escena.existe(i))
    }

    fn nombre(&self, id: Id) -> &str {
        &self.escena.objeto(id).nombre
    }

    fn padre(&self, id: Id) -> Option<Id> {
        self.escena.objeto(id).padre
    }

    // --- Aparicion (en la cámara del nivel) ------------------------------------------------------

    fn aparicion_update(&mut self, id: Id, s: &mut Aparicion, entrada: &EntradaFrame) {
        self.contador_tiempo += self.tiempo.delta;
        if entrada.escape {
            self.load_level(0);
        }
        if entrada.e_soltada {
            s.zoom = false;
        }
        if entrada.e && !s.zoom {
            // Camera.main: la cámara de la escena, que es este mismo objeto.
            if let Some(c) = self.escena.objeto_mut(id).camara.as_mut() {
                c.tamano_ortografico = if c.tamano_ortografico == 9.0 {
                    20.0
                } else {
                    9.0
                };
            }
            s.zoom = true;
        }
        s.timer += self.tiempo.delta;
        if s.timer > s.timer_max {
            // Random.Range(0, 19): la última de las 20 neuronas nunca sale.
            let i = azar_entero(0, 19) as usize;
            let x = azar_entero(-66, 19) as f32;
            let y = azar_entero(-32, 18) as f32;
            // Instantiate(null) lanza ArgumentException, que el catch (UnityException) no atrapa.
            let Some(Some(neurona)) = s.neuronas.get(i).cloned() else {
                return;
            };
            self.instanciar(&neurona, vec3(x, y, 0.0), Quat::IDENTITY);
            s.timer = 0.0;
        }
    }

    // --- BaseMovimiento y BaseRotacion (la neurona del jugador) -----------------------------------

    fn base_movimiento_update(&mut self, id: Id, velocidad: f32) {
        let v = vec3(
            self.entrada.eje("Horizontal"),
            self.entrada.eje("Vertical"),
            0.0,
        ) * velocidad
            * self.tiempo.delta;
        trasladar(&mut self.escena, id, v);
    }

    fn base_rotacion_update(&mut self, id: Id, s: &mut BaseRotacion) {
        // Rotacion: Transform.Rotate(0, 0, -(eje * velocidad * deltaTime)) en el espacio propio.
        let grados = -(self.entrada.eje("Rotacion") * s.velocidad * self.tiempo.delta);
        let obj = self.escena.objeto_mut(id);
        obj.local.rotacion *= Quat::from_rotation_z(grados.to_radians());
        if self.entrada.boton_soltado("Fire2") {
            s.up_button = true;
        }
        if self.entrada.boton("Fire2") {
            let hijos = self.escena.objeto(id).hijos.clone();
            if hijos.len() > 4 && s.up_button {
                // La última pieza unida: su centro (el primer hijo) y las patas de ese centro.
                let ultima = hijos[hijos.len() - 1];
                if let Some(&centro) = self.escena.objeto(ultima).hijos.first() {
                    let patas = self.escena.objeto(centro).hijos.clone();
                    for &pata in &patas {
                        for c in self.comportamientos.iter_mut().flatten() {
                            if let (true, Script::Union(u)) = (c.id == pata, &mut c.script) {
                                u.estado = "libre".to_owned();
                            }
                        }
                    }
                    if let Some(&pata) = patas.first() {
                        self.union_desconectar(pata);
                    }
                }
            }
            s.up_button = false;
        }
    }

    // --- Camera2DFollow -------------------------------------------------------------------------

    fn camera_follow_start(&mut self, id: Id, s: &mut Camera2DFollow) {
        let Some(target) = self.vivo(s.target) else {
            return;
        };
        let t = self.escena.posicion(target);
        s.last_target_position = t;
        s.offset_z = (self.escena.posicion(id) - t).z;
        self.escena.fijar_padre(id, None);
    }

    fn camera_follow_update(&mut self, id: Id, s: &mut Camera2DFollow) {
        let Some(target) = self.vivo(s.target) else {
            return;
        };
        let dt = self.tiempo.delta;
        let objetivo = self.escena.posicion(target);
        let x = (objetivo - s.last_target_position).x;
        if x.abs() > s.look_ahead_move_threshold {
            s.look_ahead_pos = s.look_ahead_factor * Vec3::X * signo(x);
        } else {
            s.look_ahead_pos =
                mover_hacia(s.look_ahead_pos, Vec3::ZERO, dt * s.look_ahead_return_speed);
        }
        // Vector3.forward es (0, 0, 1).
        let delante = objetivo + s.look_ahead_pos + Vec3::Z * s.offset_z;
        let actual = self.escena.posicion(id);
        let p = suavizar(
            actual,
            delante,
            &mut s.current_velocity,
            s.damping,
            f32::INFINITY,
            dt,
        );
        self.escena.fijar_posicion(id, p);
        s.last_target_position = self.escena.posicion(target);
    }

    // --- Union (las patas de las neuronas) --------------------------------------------------------

    /// Union.desconectar: el abuelo de la pata (su neurona) se queda sin padre.
    fn union_desconectar(&mut self, pata: Id) {
        if let Some(neurona) = self.padre(pata).and_then(|c| self.padre(c)) {
            self.escena.fijar_padre(neurona, None);
        }
    }

    fn union_trigger(&mut self, id: Id, s: &Union, otro: Id) {
        // transform.parent.parent: sin abuelo, NullReferenceException.
        let Some(neurona) = self.padre(id).and_then(|c| self.padre(c)) else {
            return;
        };
        let es_pata = self.nombre(otro).contains("pata");
        // «&» sin cortocircuito: el nombre de la neurona se lee siempre.
        if es_pata & self.nombre(neurona).contains("ataque") {
            for c in self.comportamientos.iter_mut().flatten() {
                if let (true, Script::Disparar(d)) = (c.id == neurona, &mut c.script) {
                    d.valido = true;
                }
            }
        }
        self.choque = true;
        if es_pata && self.padre(neurona).is_none() {
            // SetParent(other.transform.parent): la neurona pasa a colgar del centro de la otra,
            // conservando su pose en el mundo (Unity no deja armar un ciclo).
            if let Some(destino) = self.padre(otro)
                && !self.jerarquia(neurona).contains(&destino)
            {
                self.escena.fijar_padre(neurona, Some(destino));
            }
            if let Some(clip) = &s.sonido {
                self.audio.play_one_shot(clip, 100.0);
            }
        }
        self.choque = true;
    }

    // --- disparar y bala ------------------------------------------------------------------------

    fn disparar_update(&mut self, id: Id, s: &mut Disparar, entrada: &EntradaFrame) {
        if !s.valido {
            return;
        }
        if entrada.disparo_soltado {
            s.disparo = false;
        }
        if entrada.disparo && !s.disparo {
            let rotacion = self.escena.rotacion(id);
            let desplazamiento = rotacion * s.desplazamiento;
            let posicion = self.escena.posicion(id) + desplazamiento;
            if let Some(bala) = &s.bala {
                self.instanciar(bala, posicion, rotacion);
            }
            if let Some(clip) = &s.sonido {
                self.audio.play_one_shot(clip, 100.0);
            }
            s.disparo = true;
        }
    }

    fn bala_update(&mut self, id: Id) {
        // Una unidad por frame hacia su arriba.
        trasladar(&mut self.escena, id, Vec3::Y);
    }

    fn bala_trigger(&mut self, id: Id, otro: Id) {
        let nombre = self.nombre(otro);
        if nombre.contains("enemigo") || nombre.contains("lateral") || nombre.contains("esquina") {
            // En un borde no hay muerte_enemigo: el original escribe en la memoria baja del asm.js
            // y sigue.
            for c in self.comportamientos.iter_mut().flatten() {
                if let (true, Script::MuerteEnemigo { vida }) = (c.id == otro, &mut c.script) {
                    *vida -= 1.0;
                }
            }
            self.escena.destruir(id);
        }
    }

    fn bala_colision(&mut self, id: Id, otro: Id) {
        let nombre = self.nombre(otro);
        if nombre.contains("Enemy") || nombre.contains("esquina") {
            self.escena.destruir(id);
        }
    }

    fn borde_toque(&mut self, otro: Id) {
        let nombre = self.nombre(otro);
        if nombre.contains("enemigo") || nombre.contains("bala") {
            self.escena.destruir(otro);
        }
    }

    // --- El enemigo: IA_Enemy y muerte_enemigo ----------------------------------------------------

    fn muerte_trigger(&mut self, id: Id, otro: Id) {
        let nombre = self.nombre(otro);
        if nombre.contains("neurona") {
            if nombre.contains("neurona_base") {
                self.tiempo_guardado = self.contador_tiempo;
                self.load_level(GAME_OVER);
            }
            self.escena.destruir(otro);
            self.escena.destruir(id);
        }
    }

    /// IA_Enemy.CrearWaypoint: una esfera invisible (GameObject.CreatePrimitive) en un punto al
    /// azar del nivel.
    fn crear_waypoint(&mut self, s: &mut IaEnemy) {
        let datos = DatosObjeto {
            id: -1,
            nombre: "Sphere".to_owned(),
            activo: true,
            capa: 0,
            etiqueta: "Untagged".to_owned(),
            posicion: [0.0; 3],
            rotacion: [0.0, 0.0, 0.0, 1.0],
            escala: [1.0; 3],
            padre: None,
            hijos: Vec::new(),
            componentes: Vec::new(),
        };
        let esfera =
            self.escena
                .instanciar_datos(&self.proyecto, &[datos], Vec3::ZERO, Quat::IDENTITY);
        self.escena.objeto_mut(esfera).nombre = "Sphere".to_owned();
        s.waypoint = Some(esfera);
        let x = azar_entero(-66, 19) as f32;
        let y = azar_entero(-32, 18) as f32;
        self.escena.fijar_posicion(esfera, vec3(x, y, 0.0));
    }

    /// IA_Enemy.Seguir: avanza hacia el objetivo, primero en x y después en y. Mathf.Round quedó
    /// inlined como roundf (las mitades hacia afuera, como f32::round), y el signo de ±0 es 1.
    fn seguir(&mut self, id: Id, s: &IaEnemy, objetivo: Id) {
        let dt = self.tiempo.delta;
        let dx = signo((self.escena.posicion(objetivo).x - self.escena.posicion(id).x).round());
        let p = self.escena.posicion(id);
        self.escena
            .fijar_posicion(id, vec3(p.x + dx * s.velocidad_x * dt, p.y, p.z));
        let dy = signo((self.escena.posicion(objetivo).y - self.escena.posicion(id).y).round());
        let p = self.escena.posicion(id);
        self.escena
            .fijar_posicion(id, vec3(p.x, p.y + dy * s.velocidad_y * dt, p.z));
    }

    fn ia_update(&mut self, id: Id, s: &mut IaEnemy) {
        let Some(target) = self.vivo(s.target) else {
            return;
        };
        let distancia = self
            .escena
            .posicion(id)
            .distance(self.escena.posicion(target));
        let atacar = if distancia > s.distancia_target {
            match s.estado {
                2 => true,
                3 => {
                    if self.vivo(s.waypoint).is_none() {
                        self.crear_waypoint(s);
                    }
                    let Some(waypoint) = self.vivo(s.waypoint) else {
                        return;
                    };
                    // El waypoint no se renueva: al llegar, el enemigo se queda quieto.
                    if self
                        .escena
                        .posicion(id)
                        .distance(self.escena.posicion(waypoint))
                        > s.distancia_waypoint
                    {
                        self.seguir(id, s, waypoint);
                    }
                    false
                }
                1 => {
                    // Huir: nadie pone este estado; multiplica la posición entera por deltaTime.
                    let q = self.escena.posicion(id) - self.escena.posicion(target);
                    let d = q / q.length();
                    let p = self.escena.posicion(id);
                    let nueva = vec3(d.x + p.x, d.y + p.y, 0.0) * self.tiempo.delta;
                    self.escena.fijar_posicion(id, nueva);
                    false
                }
                0 => {
                    let t = self.tiempo.tiempo as f32;
                    if t >= s.tiempo_quieto_check as f32 {
                        s.tiempo_quieto_check = (t + s.tiempo_quieto as f32) as i32;
                        s.estado = 3;
                    }
                    false
                }
                _ => false,
            }
        } else {
            s.estado = 2;
            true
        };
        if atacar {
            self.seguir(id, s, target);
            if distancia <= s.distancia_muerte {
                self.escena.destruir(id);
            }
            if distancia >= s.distancia_target {
                s.estado = 0;
            }
        }
    }

    // --- GeneradorVitaminas ---------------------------------------------------------------------

    fn generador_update(&mut self, id: Id, s: &mut Generador) {
        let t = self.tiempo.tiempo as f32;
        if t < s.tiempo_lanzar_check as f32 {
            return;
        }
        let i = azar_entero(0, s.objetos.len() as i32) as usize;
        // Si el enemigo que clona ya no existe, Instantiate(null) lanza una excepción: no se
        // crean más y el tiempo no avanza.
        let Some(fuente) = s.objetos.get(i).copied().flatten() else {
            return;
        };
        if self.vivo(Some(fuente.0)).is_none() {
            return;
        }
        let posicion = self.escena.posicion(id);
        self.clonar(fuente, posicion, Quat::IDENTITY);
        let t = self.tiempo.tiempo as f32;
        s.tiempo_lanzar_check = (t + s.tiempo_lanzar as f32) as i32;
    }

    // --- Mensajes de la física ------------------------------------------------------------------

    /// OnTriggerEnter2D u OnCollisionEnter2D de los scripts de `id` con el colisionador de `otro`.
    fn mensaje(&mut self, id: Id, otro: Id, trigger: bool) {
        for i in 0..self.comportamientos.len() {
            if !self.escena.existe(id) || !self.escena.existe(otro) {
                return;
            }
            let Some(c) = self.comportamientos[i].take() else {
                continue;
            };
            if c.id == id {
                match (&c.script, trigger) {
                    (Script::Union(u), true) => self.union_trigger(id, u, otro),
                    (Script::Bala, true) => self.bala_trigger(id, otro),
                    (Script::Bala, false) => self.bala_colision(id, otro),
                    (Script::Bordescript, _) => self.borde_toque(otro),
                    (Script::MuerteEnemigo { .. }, true) => self.muerte_trigger(id, otro),
                    _ => {}
                }
            }
            self.comportamientos[i] = Some(c);
        }
    }

    fn al_tocar(&mut self, c: Contacto2D) {
        if c.entra {
            self.mensaje(c.a, c.b, c.trigger);
            self.mensaje(c.b, c.a, c.trigger);
        }
    }

    // --- El frame -------------------------------------------------------------------------------

    /// Borra lo destruido y los scripts de sus objetos.
    fn limpiar(&mut self) {
        self.escena.limpiar_destruidos();
        let escena = &self.escena;
        self.comportamientos
            .retain(|c| c.as_ref().is_some_and(|c| escena.existe(c.id)));
    }

    /// Un frame de Unity que duró `real` segundos, hasta antes de dibujar.
    pub fn frame(&mut self, real: f32, entrada: &EntradaFrame) {
        if let Some(n) = self.cargar.take() {
            self.nivel = n;
            self.escena = Escena::cargar_nivel(&self.proyecto, n);
            self.comportamientos.clear();
            let ids = self.escena.ids();
            self.crear_comportamientos(&ids);
        }
        self.tiempo.nuevo_frame(real);
        self.entrada.actualizar(&entrada.teclas, real);
        // Physics2D a paso fijo y sus mensajes (ningún script tiene FixedUpdate).
        let mut hubo_paso = false;
        while self.tiempo.siguiente_paso_fijo() {
            hubo_paso = true;
            let contactos = self.escena.paso_fisica(self.tiempo.paso_fijo);
            for c in contactos {
                self.al_tocar(c);
            }
        }
        if hubo_paso {
            self.limpiar();
        }
        // Start de los scripts nuevos y después los Update, en el orden de los objetos.
        let n = self.comportamientos.len();
        let mut orden: Vec<usize> = (0..n).collect();
        orden.sort_by_key(|&i| {
            self.comportamientos[i]
                .as_ref()
                .map_or(usize::MAX, |c| c.id)
        });
        for &i in &orden {
            let Some(mut c) = self.comportamientos[i].take() else {
                continue;
            };
            if !c.iniciado {
                c.iniciado = true;
                match &mut c.script {
                    Script::Camera2DFollow(s) => self.camera_follow_start(c.id, s),
                    Script::IaEnemy(s) => s.estado = 3,
                    _ => {}
                }
            }
            self.comportamientos[i] = Some(c);
        }
        for &i in &orden {
            let Some(mut c) = self.comportamientos[i].take() else {
                continue;
            };
            let id = c.id;
            if self.escena.existe(id) && self.escena.activo_en_jerarquia(id) {
                match &mut c.script {
                    Script::Aparicion(s) => self.aparicion_update(id, s, entrada),
                    Script::BaseMovimiento { velocidad } => {
                        let v = *velocidad;
                        self.base_movimiento_update(id, v)
                    }
                    Script::BaseRotacion(s) => self.base_rotacion_update(id, s),
                    Script::Camera2DFollow(s) => self.camera_follow_update(id, s),
                    Script::Union(_) => self.choque = false,
                    Script::Bala => self.bala_update(id),
                    Script::Disparar(s) => self.disparar_update(id, s, entrada),
                    Script::MuerteEnemigo { vida } => {
                        if *vida <= 0.0 {
                            self.escena.destruir(id);
                        }
                    }
                    Script::IaEnemy(s) => self.ia_update(id, s),
                    Script::Generador(s) => self.generador_update(id, s),
                    Script::Titulo { .. } => {
                        if entrada.e {
                            self.load_level(1);
                        }
                        // Escape: Application.Quit, que en el WebGL (el único build del original)
                        // no hace nada.
                    }
                    Script::Intrucciones { .. } | Script::GameOver { .. } => {
                        if entrada.e {
                            self.load_level(NIVEL);
                        }
                        if entrada.escape {
                            self.load_level(0);
                        }
                    }
                    Script::AnimacionUnion | Script::Bordescript => {}
                }
            }
            self.comportamientos[i] = Some(c);
        }
        self.animar(self.tiempo.delta);
        self.limpiar();
    }

    /// El Animator (todos en modo Normal): el sprite de cada objeto animado.
    fn animar(&mut self, dt: f32) {
        for id in self.escena.ids() {
            if !self.escena.activo_en_jerarquia(id) {
                continue;
            }
            let obj = self.escena.objeto_mut(id);
            let Some(a) = obj.animator.as_mut() else {
                continue;
            };
            if !a.activo {
                continue;
            }
            let sprite = a.actualizar(&self.proyecto.clips, dt);
            if let (Some(sprite), Some(s)) = (sprite, obj.sprite.as_mut()) {
                s.sprite = Some(sprite);
            }
        }
    }

    fn estilo(&self, nombre: &str) -> Option<Estilo> {
        self.proyecto.estilos.get(nombre).cloned()
    }

    /// El dibujo del frame y OnGUI.
    pub fn dibujar(&mut self, recursos: &Recursos, gui: &mut Gui) {
        let camara = self
            .escena
            .ids()
            .into_iter()
            .find(|&id| self.escena.objeto(id).camara.is_some());
        if let Some(c) = camara {
            let camara = Camara::de(&self.escena, c, ANCHO, ALTO);
            render::dibujar(&self.escena, &self.proyecto, recursos, &camara);
        }
        gui.empezar(unity::gui::Evento::Repaint);
        let pantalla = Rect::new(0.0, 0.0, ANCHO, ALTO);
        let texto_puntaje = format!(
            "TU PUNTAJE FUE DE {} SEGUNDOS",
            // Single.ToString("0"): redondea las mitades hacia afuera.
            self.tiempo_guardado.round() as i64
        );
        let mut pedidos: Vec<(String, Option<String>)> = Vec::new();
        for c in self.comportamientos.iter().flatten() {
            match &c.script {
                Script::Titulo { textura } | Script::Intrucciones { textura } => {
                    pedidos.push((textura.clone(), None))
                }
                Script::GameOver { textura, estilo } => {
                    pedidos.push((textura.clone(), Some(estilo.clone())))
                }
                _ => {}
            }
        }
        for (textura, estilo) in pedidos {
            // GUI.DrawTexture(new Rect(0, 0, Screen.width, Screen.height), aTexture).
            if let Some(t) = recursos.texturas.get(&textura) {
                gui.dibujar_textura(pantalla, t);
            }
            if let Some(e) = estilo.and_then(|e| self.estilo(&e)) {
                gui.label(Rect::new(0.0, 20.0, ANCHO, 100.0), &texto_puntaje, &e);
            }
        }
        gui.presentar();
    }
}
