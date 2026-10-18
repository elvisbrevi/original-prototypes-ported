// Port de Engine.Objetos: Objeto y sus subclases (Grafico, Plataforma, Personaje,
// SistemaParticulas y Texto), Estado (animación) y Particula. Las subclases solo cambian cargar,
// actualizar y dibujar, así que aquí son un único tipo con `tipo`.

use macroquad::prelude::*;
use xna::{Efectos, Random, Rectangle, SEGUNDOS_POR_PASO, SEGUNDOS_POR_PASO_F64, int};

use crate::assets::Assets;
use crate::audio::Sonido;
use crate::motor::shader::Materiales;
use crate::motor::{Globales, Post, Shader};

/// Objeto.Tipo. El valor por defecto de la enumeración es Personaje.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tipo {
    #[default]
    Personaje,
    Grafico,
    Plataforma,
    Particulas,
    Texto,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Colision {
    #[default]
    Ninguna,
    Total,
    Lateral,
    Superior,
    Inferior,
    /// Existe en el motor, pero ningún mapa la usa y la física no la trata.
    Pixel,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lugar {
    #[default]
    Fondo,
    Medio,
    Frente,
}

/// SpriteBlendMode: AlphaBlend o Additive.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mezcla {
    #[default]
    Alfa,
    Aditiva,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Variacion {
    #[default]
    Ninguna,
    Shader,
    Rotacion,
    Tamano,
}

/// Engine.Objetos.Estado: una animación de la hoja de sprites.
#[derive(Clone, Debug)]
pub struct Estado {
    pub cuadro: Rectangle,
    pub variacion: Variacion,
    pub fps: f32,
    pub frama: f32,
    pub numero: f32,
    pub min_variacion: f32,
    pub max_variacion: f32,
    pub velocidad_variacion: f32,
    pub no_repetir: bool,
    pub variar_mas: bool,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            cuadro: Rectangle::default(),
            variacion: Variacion::Ninguna,
            fps: 0.0,
            frama: 0.0,
            numero: 0.0,
            min_variacion: 0.0,
            max_variacion: 0.0,
            velocidad_variacion: 0.0,
            no_repetir: false,
            variar_mas: true,
        }
    }
}

/// Engine.Objetos.Particula.
#[derive(Clone, Copy)]
struct Particula {
    posicion: Vec2,
    direccion: Vec2,
    angulo: f32,
    velocidad_angular: f32,
    color: Color,
    tamano: f32,
    tiempo_vida: i32,
}

pub struct Objeto {
    pub tipo: Tipo,
    pub nombre: String,
    pub texto: String,
    pub nombre_textura: String,
    pub direccion: Vec2,
    pub colision: Colision,
    pub mezcla: Mezcla,
    pub movimento: bool,
    pub animado: bool,
    pub gravedad: bool,
    pub colision_superior: bool,
    pub colision_inferior: bool,
    pub colision_derecha: bool,
    pub colision_izquierda: bool,
    pub repetir_scroll: bool,
    pub fuera_de_camara: bool,
    pub no_shader_camara: bool,
    pub scroll_horizontal: f32,
    pub scroll_vertical: f32,
    pub tamano: f32,
    pub inclinacion: f32,
    pub fuerza_gravedad: f32,
    pub velocidad: f32,
    pub estado_seleccionado: usize,
    pub cantidad_particulas: i32,
    pub posicion: Vec2,
    pub destino: Vec2,
    pub inicio: Vec2,
    pub origen: Vec2,
    /// Color_ como bytes de C# (las restas dan la vuelta).
    pub color: [u8; 4],
    pub efectos: Efectos,
    pub lugar: Lugar,
    pub cuerpo: Rectangle,
    pub estados: Vec<Estado>,
    pub sonidos: Vec<Sonido>,
    pub shader: Shader,
    pub post: Option<Post>,
    pub respawn: f32,
    pub ida_x: bool,
    pub ida_y: bool,
    fade_in: bool,
    fade_out: bool,
    aditivo: i32,
    pub color_random: bool,
    pub tamano_random: bool,
    pub direccion_random: bool,
    pub activo: bool,
    /// Tamaño de la textura (o 0 si es un Texto).
    textura: Vec2,
    /// Grafico: posición en pantalla tras aplicar el scroll.
    x: f32,
    y: f32,
    particulas: Vec<Particula>,
    random: Option<Random>,
}

impl Default for Objeto {
    fn default() -> Self {
        Self {
            tipo: Tipo::default(),
            nombre: String::new(),
            texto: String::new(),
            nombre_textura: String::new(),
            direccion: Vec2::ZERO,
            colision: Colision::Ninguna,
            mezcla: Mezcla::Alfa,
            movimento: false,
            animado: false,
            gravedad: false,
            colision_superior: false,
            colision_inferior: false,
            colision_derecha: false,
            colision_izquierda: false,
            repetir_scroll: false,
            fuera_de_camara: false,
            no_shader_camara: false,
            scroll_horizontal: 0.0,
            scroll_vertical: 0.0,
            tamano: 0.0,
            inclinacion: 0.0,
            fuerza_gravedad: 0.0,
            velocidad: 0.0,
            estado_seleccionado: 0,
            cantidad_particulas: 0,
            posicion: Vec2::ZERO,
            destino: Vec2::ZERO,
            inicio: Vec2::ZERO,
            origen: Vec2::ZERO,
            color: [0; 4],
            efectos: Efectos::NINGUNO,
            lugar: Lugar::Fondo,
            cuerpo: Rectangle::default(),
            estados: Vec::new(),
            sonidos: Vec::new(),
            shader: Shader::Ninguno,
            post: None,
            respawn: 0.0,
            ida_x: true,
            ida_y: true,
            fade_in: false,
            fade_out: true,
            aditivo: 0,
            color_random: false,
            tamano_random: false,
            direccion_random: false,
            activo: false,
            textura: Vec2::ZERO,
            x: 0.0,
            y: 0.0,
            particulas: Vec::new(),
            random: None,
        }
    }
}

/// Paso de los fundidos: ElapsedGameTime.TotalSeconds * 100 * velocidad.
fn paso_fundido(velocidad: f64) -> f64 {
    SEGUNDOS_POR_PASO_F64 * 100.0 * velocidad
}

impl Objeto {
    pub fn color(&self) -> Color {
        let [r, g, b, a] = self.color;
        Color::from_rgba(r, g, b, a)
    }

    /// cargar de cada subclase (más cargarSonidos, que se hace al leer el mapa).
    pub fn cargar(&mut self, assets: &Assets) {
        match self.tipo {
            Tipo::Texto => {}
            Tipo::Grafico => {
                self.textura = assets.textura(&self.nombre_textura).size();
                self.origen = self.textura / (2.0 * self.tamano);
            }
            Tipo::Particulas => {
                self.textura = assets.textura(&self.nombre_textura).size();
                self.particulas = Vec::new();
                self.random = Some(Random::con_reloj());
            }
            Tipo::Personaje | Tipo::Plataforma => {
                self.textura = assets.textura(&self.nombre_textura).size();
            }
        }
    }

    /// actualizar de cada subclase.
    pub fn actualizar(&mut self, glob: &Globales) {
        match self.tipo {
            Tipo::Grafico => {
                self.actualizar_cuerpo(vec2(self.x, self.y));
                self.scroll(glob);
                if self.movimento {
                    self.mover();
                }
                if self.animado && !self.estados.is_empty() {
                    self.animar();
                }
            }
            Tipo::Personaje => {
                self.actualizar_cuerpo(self.posicion);
                if self.movimento {
                    self.mover();
                }
                if self.animado && !self.estados.is_empty() {
                    self.animar();
                }
            }
            Tipo::Texto => {
                if self.movimento {
                    self.mover();
                }
                if self.animado && !self.estados.is_empty() {
                    self.animar();
                }
            }
            Tipo::Plataforma => {
                self.cuerpo = Rectangle::new(
                    int(self.posicion.x),
                    int(self.posicion.y),
                    self.cuerpo.width,
                    self.cuerpo.height,
                );
                if self.movimento {
                    self.mover();
                }
            }
            Tipo::Particulas => self.actualizar_particulas(),
        }
    }

    /// Objeto.actualizarSonidos: suenan los activados y se desactivan.
    pub fn actualizar_sonidos(&mut self) {
        for sonido in &mut self.sonidos {
            sonido.play();
            sonido.activo = false;
        }
    }

    /// actualizarCuerpo de Grafico (con su posición tras el scroll del paso anterior) y Personaje.
    fn actualizar_cuerpo(&mut self, esquina: Vec2) {
        let tamano = if let Some(e) = self.estados.get(self.estado_seleccionado) {
            vec2(e.cuadro.width as f32, e.cuadro.height as f32)
        } else {
            self.textura
        };
        self.cuerpo = Rectangle::new(
            int(esquina.x),
            int(esquina.y),
            int(tamano.x * self.tamano),
            int(tamano.y * self.tamano),
        );
        // Origen con división entera.
        self.origen = vec2((tamano.x as i32 / 2) as f32, (tamano.y as i32 / 2) as f32);
    }

    /// Grafico.scroll: parallax respecto a la pantalla y repetición de fondos.
    fn scroll(&mut self, glob: &Globales) {
        let p = &glob.pantalla;
        self.x = self.posicion.x + (0.0 - p.posicion.x) * self.scroll_horizontal;
        self.y = self.posicion.y + (0.0 - p.posicion.y) * self.scroll_vertical;
        let c = self.cuerpo;
        if !self.movimento {
            if self.repetir_scroll && (c.right() as f32) < p.posicion.x {
                self.posicion.x = p.posicion.x + p.v_ancho as f32;
            }
            if self.repetir_scroll && (c.left() as f32) > p.posicion.x + p.v_ancho as f32 {
                self.posicion.x = p.posicion.x - c.width as f32;
            }
            if self.repetir_scroll && (c.bottom() as f32) < p.posicion.y {
                self.posicion.y = p.posicion.y + p.v_alto as f32;
            }
            if self.repetir_scroll && (c.top() as f32) > p.posicion.y + p.v_alto as f32 {
                self.posicion.y = p.posicion.y - c.height as f32;
            }
        } else {
            if self.repetir_scroll
                && c.left() as f32 >= self.destino.x
                && self.destino.x != self.inicio.x
            {
                self.posicion.x = self.inicio.x - c.width as f32;
            }
            if self.repetir_scroll
                && c.top() as f32 >= self.destino.y
                && self.destino.y != self.inicio.y
            {
                self.posicion.y = self.inicio.y - c.height as f32;
            }
        }
    }

    /// Objeto.mover: va y vuelve entre Inicio y Destino.
    fn mover(&mut self) {
        if self.inicio.x != self.destino.x {
            if self.ida_x {
                self.posicion.x += self.velocidad;
                if self.posicion.x > self.destino.x {
                    self.ida_x = false;
                }
            } else {
                self.posicion.x -= self.velocidad;
                if self.posicion.x < self.inicio.x {
                    self.ida_x = true;
                }
            }
        }
        if self.inicio.y == self.destino.y {
            return;
        }
        if self.ida_y {
            self.posicion.y += self.velocidad;
            if self.posicion.y > self.destino.y {
                self.ida_y = false;
            }
        } else {
            self.posicion.y -= self.velocidad;
            if self.posicion.y < self.inicio.y {
                self.ida_y = true;
            }
        }
    }

    /// Objeto.cambiarEstado.
    pub fn cambiar_estado(&mut self, indice: usize) {
        self.estado_seleccionado = indice;
        if self.estados[indice].numero > 1.0 {
            self.estados[indice].cuadro.x = 0;
        }
    }

    /// Estado.animar: avanza los cuadros de la hoja y aplica la variación.
    fn animar(&mut self) {
        let i = self.estado_seleccionado;
        if self.estados[i].variacion != Variacion::Ninguna {
            self.aplicar_variacion(i);
        }
        let e = &mut self.estados[i];
        if e.frama > e.cuadro.width as f32 {
            e.frama = 0.0;
            let ultimo = e.cuadro.width as f32 * e.numero - e.cuadro.width as f32;
            if !e.no_repetir {
                if e.cuadro.x as f32 >= ultimo {
                    e.cuadro.x = -e.cuadro.width;
                }
                e.cuadro.x += e.cuadro.width;
            } else if (e.cuadro.x as f32) < ultimo {
                e.cuadro.x += e.cuadro.width;
            }
        }
        e.frama += e.fps * (SEGUNDOS_POR_PASO * 10.0);
    }

    /// Estado.aplicarVariacion (los mapas solo usan la de rotación).
    fn aplicar_variacion(&mut self, i: usize) {
        let e = &mut self.estados[i];
        let valor = match e.variacion {
            Variacion::Rotacion => &mut self.inclinacion,
            Variacion::Tamano => &mut self.tamano,
            Variacion::Shader => match (self.shader, &mut self.post) {
                (Shader::Brightness, Some(post)) => &mut post.intensidad,
                _ => return,
            },
            Variacion::Ninguna => return,
        };
        if e.variacion == Variacion::Rotacion && e.max_variacion == 0.0 && e.min_variacion == 0.0 {
            *valor += e.velocidad_variacion;
        } else if e.variar_mas {
            *valor += e.velocidad_variacion;
            if *valor > e.max_variacion {
                e.variar_mas = false;
            }
        } else {
            *valor -= e.velocidad_variacion;
            if *valor < e.min_variacion {
                e.variar_mas = true;
            }
        }
    }

    /// Objeto.Fade: parpadeo del alfa entre 0 y 254.
    pub fn fade(&mut self, velocidad: u8) {
        let paso = paso_fundido(velocidad as f64);
        let a = &mut self.color[3];
        if self.fade_out {
            if *a as f64 > paso {
                *a = a.wrapping_sub(paso as u8);
            } else {
                self.fade_out = false;
                self.fade_in = true;
            }
        }
        if self.fade_in {
            if (*a as f64) < 254.0 - paso {
                *a = a.wrapping_add(paso as u8);
                return;
            }
            self.fade_in = false;
            self.fade_out = true;
        }
    }

    pub fn fade_out(&mut self, velocidad: u8) {
        let paso = paso_fundido(velocidad as f64);
        let a = &mut self.color[3];
        if *a as f64 > paso {
            *a = a.wrapping_sub(paso as u8);
            return;
        }
        *a = 0;
        self.activo = false;
    }

    pub fn fade_in(&mut self, velocidad: u8) {
        if !self.activo {
            self.activo = true;
        }
        let paso = paso_fundido(velocidad as f64);
        let a = &mut self.color[3];
        if (*a as f64) < 254.0 - paso {
            *a = a.wrapping_add(paso as u8);
        } else {
            *a = 255;
        }
    }

    /// Objeto.atenuarAditivo: alterna la mezcla aditiva y la normal.
    pub fn atenuar_aditivo(&mut self, velocidad: i32) {
        let paso = paso_fundido(velocidad as f64);
        if self.mezcla == Mezcla::Aditiva {
            if self.aditivo as f64 > paso {
                self.aditivo -= paso as u8 as i32;
            } else {
                self.mezcla = Mezcla::Alfa;
            }
        }
        if self.mezcla == Mezcla::Alfa {
            if (self.aditivo as f64) < paso {
                self.aditivo += paso as u8 as i32;
            } else {
                self.mezcla = Mezcla::Aditiva;
            }
        }
    }

    /// Objeto.reproducir(nombre).
    pub fn reproducir(&mut self, nombre: &str) {
        for s in self.sonidos.iter_mut().filter(|s| s.nombre == nombre) {
            s.activo = true;
        }
    }

    /// Objeto.reproducir(nombre, emisor, oyente): el volumen baja con la distancia (en
    /// coordenadas de manzana: |dx| + |dy|).
    pub fn reproducir_desde(&mut self, nombre: &str, emisor: Vec2, oyente: Vec2) {
        let distancia = ((emisor.x - oyente.x).abs() + (emisor.y - oyente.y).abs()) * 0.001;
        for s in self.sonidos.iter_mut().filter(|s| s.nombre == nombre) {
            let volumen = s.volumen - distancia;
            let volumen = if (0.0..=1.0).contains(&volumen) {
                volumen
            } else {
                0.0
            };
            s.instancia.borrow_mut().set_volumen(volumen);
            s.activo = true;
        }
    }

    pub fn stop(&mut self, nombre: &str) {
        for s in self.sonidos.iter_mut().filter(|s| s.nombre == nombre) {
            s.activo = false;
        }
    }

    // --- SistemaParticulas -------------------------------------------------------------------

    fn actualizar_particulas(&mut self) {
        if self.movimento {
            self.mover();
        }
        for _ in 0..self.cantidad_particulas {
            if self.activo {
                let p = self.generar_nueva_particula();
                self.particulas.push(p);
            }
        }
        let mut j = 0;
        while j < self.particulas.len() {
            let p = &mut self.particulas[j];
            p.tiempo_vida -= 1;
            p.posicion += p.direccion;
            p.angulo += p.velocidad_angular;
            if p.tiempo_vida <= 0 {
                self.particulas.remove(j);
            } else {
                j += 1;
            }
        }
    }

    fn generar_nueva_particula(&mut self) -> Particula {
        let random = self.random.as_mut().expect("sistema de partículas cargado");
        if self.direccion_random {
            self.direccion = vec2(
                (random.next_double() * 2.0 - 1.0) as f32,
                (random.next_double() * 2.0 - 1.0) as f32,
            );
        }
        let velocidad_angular = 0.1 * (random.next_double() * 2.0 - 1.0) as f32;
        if self.color_random {
            // new Color(float r, float g, float b): alfa 255, componentes redondeados.
            let componente = |v: f64| (v as f32 * 255.0).round_ties_even() as u8;
            self.color = [
                componente(random.next_double()),
                componente(random.next_double()),
                componente(random.next_double()),
                255,
            ];
        }
        let tiempo_vida = 20 + random.next(0, 40);
        Particula {
            posicion: self.posicion,
            direccion: self.direccion,
            angulo: 0.0,
            velocidad_angular,
            color: self.color(),
            tamano: self.tamano,
            tiempo_vida,
        }
    }

    // --- dibujar --------------------------------------------------------------------------------

    /// dibujar de cada subclase: un SpriteBatch.Begin propio con su mezcla, con o sin la matriz de
    /// la cámara, y el efecto de la escena o el del objeto.
    pub fn dibujar(
        &self,
        glob: &mut Globales,
        assets: &Assets,
        pantalla: &museo::Pantalla,
        materiales: &Materiales,
    ) {
        if self.fuera_de_camara {
            pantalla.vista(Vec2::ZERO);
        } else {
            pantalla.vista_transformada(glob.pantalla.matriz());
        }
        // Plataforma no dibuja nada (solo sus cuerpos, en el modo de depuración del editor).
        if self.tipo == Tipo::Plataforma {
            return;
        }
        let efecto =
            if glob.activar_shader && glob.shader != Shader::Ninguno && !self.no_shader_camara {
                glob.post
            } else if self.shader != Shader::Ninguno && !glob.activar_shader {
                self.post
            } else {
                None
            };
        materiales.usar(self.mezcla, efecto);
        match self.tipo {
            Tipo::Grafico | Tipo::Personaje => {
                let esquina = if self.tipo == Tipo::Grafico {
                    vec2(self.x, self.y)
                } else {
                    self.posicion
                };
                let fuente = self.estados.get(self.estado_seleccionado).map(|e| e.cuadro);
                xna::dibujar(
                    assets.textura(&self.nombre_textura),
                    esquina + self.origen * self.tamano,
                    fuente,
                    self.color(),
                    self.inclinacion,
                    self.origen,
                    self.tamano,
                    self.efectos,
                );
            }
            Tipo::Texto => assets.fuente(&self.nombre_textura).dibujar_ex(
                &self.texto,
                self.posicion,
                self.color(),
                self.inclinacion,
                Vec2::ZERO,
                self.tamano,
                self.efectos,
            ),
            Tipo::Particulas => {
                let textura = assets.textura(&self.nombre_textura);
                let origen = vec2(
                    (textura.width() as i32 / 2) as f32,
                    (textura.height() as i32 / 2) as f32,
                );
                for p in &self.particulas {
                    xna::dibujar(
                        textura,
                        p.posicion,
                        None,
                        p.color,
                        p.angulo,
                        origen,
                        p.tamano,
                        Efectos::NINGUNO,
                    );
                }
            }
            Tipo::Plataforma => {}
        }
        gl_use_default_material();
    }
}
