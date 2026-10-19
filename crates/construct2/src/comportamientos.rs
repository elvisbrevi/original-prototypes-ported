// Los comportamientos (behaviors) de Construct 2 que usan los juegos: el estado de cada uno y lo
// que no necesita ver otras instancias. Sus ticks con colisiones (Bullet, Turret, Wrap) están en
// mundo.rs, que tiene a todas las instancias.

use serde_json::Value;

use crate::datos::Clase;
use crate::geometria::{a_radianes, angulo_hacia, distancia};

fn prop(p: &Value, i: usize) -> f64 {
    p[i].as_f64().unwrap_or_default()
}

#[derive(Clone, Debug)]
pub enum Comportamiento {
    Plataforma(Plataforma),
    Bala(Bala),
    Torreta(Torreta),
    Movimiento(Movimiento),
    Envolver(Envolver),
    Solido(Solido),
    Fisica(Fisica),
    Desplazar(Desplazar),
    /// Un comportamiento que el port no usa.
    Otro,
}

impl Comportamiento {
    /// onCreate del comportamiento con las propiedades de la instancia.
    pub fn nuevo(clase: Option<Clase>, p: &Value, x: f64, y: f64, angulo: f64) -> Self {
        match clase {
            Some(Clase::Plataforma) => Self::Plataforma(Plataforma::nueva(p)),
            Some(Clase::Bala) => Self::Bala(Bala::nueva(p, x, y, angulo)),
            Some(Clase::Torreta) => Self::Torreta(Torreta::nueva(p)),
            Some(Clase::Movimiento) => Self::Movimiento(Movimiento::nuevo(p)),
            Some(Clase::Envolver) => Self::Envolver(Envolver {
                modo: prop(p, 0) as i32,
            }),
            Some(Clase::Solido) => Self::Solido(Solido {
                habilitado: prop(p, 0) != 0.0,
            }),
            Some(Clase::Fisica) => Self::Fisica(Fisica::nueva(p, x, y, angulo)),
            Some(Clase::Desplazar) => Self::Desplazar(Desplazar {
                habilitado: prop(p, 0) != 0.0,
            }),
            None => Self::Otro,
        }
    }
}

/// Comportamiento Platform (behaviors/platform). Sin sólidos en el proyecto nunca hay suelo, así
/// que de su posttick solo queda la caída: dy += gravedad·dt (hasta la velocidad máxima) y
/// y += dy·dt.
#[derive(Clone, Debug)]
pub struct Plataforma {
    pub gravedad: f64,
    pub caida_maxima: f64,
    pub dy: f64,
    pub habilitada: bool,
}

impl Plataforma {
    /// onCreate: propiedades [velocidad máxima, aceleración, deceleración, salto, gravedad,
    /// caída máxima, doble salto, sostener salto, controles por defecto, habilitada].
    pub fn nueva(p: &Value) -> Self {
        Self {
            gravedad: prop(p, 4),
            caida_maxima: prop(p, 5),
            dy: 0.0,
            habilitada: prop(p, 9) != 0.0,
        }
    }

    pub fn posttick(&mut self, y: &mut f64, dt: f64) {
        if !self.habilitada {
            return;
        }
        self.dy += self.gravedad * dt;
        if self.dy > self.caida_maxima {
            self.dy = self.caida_maxima;
        }
        if self.dy != 0.0 {
            *y += self.dy * dt;
        }
    }

    /// IsJumping: se mueve hacia arriba.
    pub fn subiendo(&self) -> bool {
        self.dy < 0.0
    }

    /// IsFalling.
    pub fn cayendo(&self) -> bool {
        self.dy > 0.0
    }
}

/// Comportamiento Bullet: se mueve en línea recta (con aceleración y gravedad) y puede rebotar en
/// los sólidos.
#[derive(Clone, Debug)]
pub struct Bala {
    pub dx: f64,
    pub dy: f64,
    pub aceleracion: f64,
    pub gravedad: f64,
    pub rebota: bool,
    /// "Set angle": el Sprite mira hacia donde se mueve.
    pub fijar_angulo: bool,
    pub ultima_x: f64,
    pub ultima_y: f64,
    pub ultimo_angulo: f64,
    pub recorrido: f64,
    pub habilitada: bool,
}

impl Bala {
    /// onCreate: [velocidad, aceleración, gravedad, rebotar en sólidos, fijar ángulo, habilitada].
    pub fn nueva(p: &Value, x: f64, y: f64, angulo: f64) -> Self {
        let v = prop(p, 0);
        Self {
            dx: angulo.cos() * v,
            dy: angulo.sin() * v,
            aceleracion: prop(p, 1),
            gravedad: prop(p, 2),
            rebota: prop(p, 3) != 0.0,
            fijar_angulo: prop(p, 4) != 0.0,
            ultima_x: x,
            ultima_y: y,
            ultimo_angulo: angulo,
            recorrido: 0.0,
            habilitada: prop(p, 5) != 0.0,
        }
    }

    /// La velocidad (CompareSpeed).
    pub fn velocidad(&self) -> f64 {
        distancia(0.0, 0.0, self.dx, self.dy)
    }

    /// SetSpeed: conserva la dirección (con velocidad 0 la dirección pasa a ser 0, hacia la
    /// derecha).
    pub fn set_velocidad(&mut self, v: f64) {
        let a = angulo_hacia(0.0, 0.0, self.dx, self.dy);
        self.dx = a.cos() * v;
        self.dy = a.sin() * v;
    }

    /// SetAngleOfMotion, en grados.
    pub fn set_angulo_movimiento(&mut self, grados: f64) {
        let a = a_radianes(grados);
        let v = self.velocidad();
        self.dx = a.cos() * v;
        self.dy = a.sin() * v;
    }
}

/// Comportamiento Turret: busca un objetivo en su alcance y gira hacia él. Los disparadores "On
/// shoot" y "On target acquired" y la puntería predictiva no se portaron (ningún juego los usa).
#[derive(Clone, Debug)]
pub struct Torreta {
    pub alcance: f64,
    pub rotar: bool,
    /// Radianes por segundo.
    pub velocidad_giro: f64,
    /// 0: el primero en alcance; 1: el más cercano.
    pub modo: i32,
    pub habilitada: bool,
    pub ultimo_chequeo: f64,
    /// uid del objetivo.
    pub objetivo: Option<u32>,
}

impl Torreta {
    /// onCreate: [alcance, cadencia, rotar, velocidad de giro (grados/s), modo, predictiva,
    /// velocidad del proyectil, habilitada, celdas de colisión].
    pub fn nueva(p: &Value) -> Self {
        Self {
            alcance: prop(p, 0),
            rotar: prop(p, 2) != 0.0,
            velocidad_giro: a_radianes(prop(p, 3)),
            modo: prop(p, 4) as i32,
            habilitada: prop(p, 7) != 0.0,
            ultimo_chequeo: 0.0,
            objetivo: None,
        }
    }
}

/// Comportamiento Custom Movement: velocidad propia en px/s que solo cambian los eventos.
#[derive(Clone, Debug)]
pub struct Movimiento {
    pub dx: f64,
    pub dy: f64,
    pub habilitado: bool,
}

impl Movimiento {
    /// onCreate: [modo de pasos, px por paso, habilitado]. Los modos por pasos solo añaden el
    /// disparador "On step" (que ningún juego usa): la posición final es la misma.
    pub fn nuevo(p: &Value) -> Self {
        Self {
            dx: 0.0,
            dy: 0.0,
            habilitado: prop(p, 2) != 0.0,
        }
    }

    /// tick.
    pub fn tick(&self, x: &mut f64, y: &mut f64, dt: f64) {
        if (self.dx == 0.0 && self.dy == 0.0) || !self.habilitado {
            return;
        }
        *x += self.dx * dt;
        *y += self.dy * dt;
    }

    /// SetSpeed: 0 la velocidad total (conserva la dirección), 1 la horizontal, 2 la vertical.
    pub fn set_velocidad(&mut self, cual: i32, v: f64) {
        match cual {
            0 => {
                let a = self.dy.atan2(self.dx);
                self.dx = a.cos() * v;
                self.dy = a.sin() * v;
            }
            1 => self.dx = v,
            2 => self.dy = v,
            _ => {}
        }
    }
}

/// Comportamiento Wrap: al salir por un borde aparece por el contrario.
#[derive(Clone, Debug)]
pub struct Envolver {
    /// 0: los bordes del layout; 1: los de la vista.
    pub modo: i32,
}

/// Comportamiento Solid.
#[derive(Clone, Debug)]
pub struct Solido {
    pub habilitado: bool,
}

/// Comportamiento ScrollTo: el scroll del layout sigue a la media de sus instancias (después de
/// los eventos). El temblor (Shake) no se portó.
#[derive(Clone, Debug)]
pub struct Desplazar {
    pub habilitado: bool,
}

/// Cómo se creó una articulación de Physics, para rehacerla si se rehace el cuerpo.
#[derive(Clone, Copy, Debug)]
pub enum RegistroArticulacion {
    /// Create revolute joint (punto de imagen, uid del otro objeto).
    Revolucion { punto: i32, objetivo: u32 },
    /// Create limited revolute joint (límites en grados).
    RevolucionLimitada {
        punto: i32,
        objetivo: u32,
        inferior: f64,
        superior: f64,
    },
}

/// Comportamiento Physics: un cuerpo de Box2D (crates/box2d) que manda sobre la posición y el
/// ángulo de la instancia.
#[derive(Clone, Debug)]
pub struct Fisica {
    pub inmovil: bool,
    /// Máscara de colisión: 0 el polígono de colisión, 1 la caja, 2 un círculo.
    pub mascara: i32,
    pub sin_rotacion: bool,
    pub densidad: f64,
    pub friccion: f64,
    pub elasticidad: f64,
    pub amortiguacion_lineal: f64,
    pub amortiguacion_angular: f64,
    pub bala: bool,
    pub habilitada: bool,
    pub cuerpo: Option<box2d::BodyId>,
    /// Posición, ángulo y tamaño que dejó el último tick (para ver si los eventos la movieron).
    pub ultima_x: f64,
    pub ultima_y: f64,
    pub ultimo_angulo: f64,
    pub ultimo_ancho: f64,
    pub ultimo_alto: f64,
    pub ultima_anim: Option<usize>,
    pub ultimo_cuadro: i64,
    /// La instancia se movió por fuera de la física en el tick anterior.
    pub movida: bool,
    /// Articulaciones creadas por esta instancia y cómo se crearon.
    pub articulaciones: Vec<box2d::JointId>,
    pub registros: Vec<RegistroArticulacion>,
    /// Instancias que tienen articulaciones hacia esta (uid).
    pub unidas: Vec<u32>,
}

impl Fisica {
    /// onCreate: [inmóvil, máscara de colisión, sin rotación, densidad, fricción, elasticidad,
    /// amortiguación lineal, amortiguación angular, bala, habilitada].
    pub fn nueva(p: &Value, x: f64, y: f64, angulo: f64) -> Self {
        Self {
            inmovil: prop(p, 0) != 0.0,
            mascara: prop(p, 1) as i32,
            sin_rotacion: prop(p, 2) != 0.0,
            densidad: prop(p, 3),
            friccion: prop(p, 4),
            elasticidad: prop(p, 5),
            amortiguacion_lineal: prop(p, 6),
            amortiguacion_angular: prop(p, 7),
            bala: prop(p, 8) != 0.0,
            habilitada: prop(p, 9) != 0.0,
            cuerpo: None,
            ultima_x: x,
            ultima_y: y,
            ultimo_angulo: angulo,
            ultimo_ancho: 0.0,
            ultimo_alto: 0.0,
            ultima_anim: None,
            ultimo_cuadro: -1,
            movida: false,
            articulaciones: Vec::new(),
            registros: Vec::new(),
            unidas: Vec::new(),
        }
    }
}
