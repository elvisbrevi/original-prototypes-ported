// Port de EAAE.Niveles: Nivel (la lógica común del juego) y Titulo, Nivel1, Nivel2, Nivel3 y
// Nivel4. Los objetos se buscan por nombre en el escenario cada vez, como en el original.

use macroquad::prelude::*;
use xna::{Efectos, SEGUNDOS_POR_PASO, clamp};

use crate::assets::Assets;
use crate::audio::Banco;
use crate::control::Control;
use crate::motor::escenario::Escenario;
use crate::motor::objeto::{Colision, Lugar};
use crate::motor::shader::Materiales;
use crate::motor::{Globales, Shader};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Titulo,
    Nivel1,
    Nivel2,
    Nivel3,
    Nivel4,
}

/// Lo que un nivel necesita de Game1 mientras se actualiza. El nivel activo se saca de la lista
/// para actualizarlo; `otros` tiene el resto (con None en su lugar).
pub struct Contexto<'a> {
    pub propio: usize,
    pub nivel_activo: &'a mut usize,
    pub otros: &'a mut [Option<Nivel>],
    pub glob: &'a mut Globales,
    pub assets: &'a Assets,
    pub banco: &'a mut Banco,
}

pub struct Nivel {
    tipo: Tipo,
    fin: bool,
    saltando: bool,
    dano: bool,
    distancia: i32,
    mapas: i32,
    hunters: i32,
    kamikazes: i32,
    pajaros: i32,
    bloques: i32,
    bateria: i32,
    pilas: i32,
    contador_mapas: i32,
    balas: i32,
    fachadas: i32,
    paredes: i32,
    huevos: i32,
    pub escenario: Escenario,
}

/// Color.Yellow y Color.Gray de XNA.
const AMARILLO: [u8; 4] = [255, 255, 0, 255];
const GRIS: [u8; 4] = [128, 128, 128, 255];

/// Nivel.angulo: Math.Atan2(a.Y - b.Y, a.X - b.X).
fn angulo(a: Vec2, b: Vec2) -> f32 {
    ((a.y - b.y) as f64).atan2((a.x - b.x) as f64) as f32
}

impl Nivel {
    pub fn nuevo(tipo: Tipo) -> Self {
        Self {
            tipo,
            fin: false,
            saltando: false,
            dano: false,
            distancia: 0,
            mapas: 0,
            hunters: 0,
            kamikazes: 0,
            pajaros: 0,
            bloques: 0,
            bateria: 0,
            pilas: 0,
            contador_mapas: 0,
            balas: 0,
            fachadas: 0,
            paredes: 0,
            huevos: 0,
            escenario: Escenario::default(),
        }
    }

    /// cargar de cada nivel: su mapa y, salvo en el título, contarObjetos.
    pub fn cargar(&mut self, glob: &mut Globales, assets: &Assets, banco: &mut Banco) {
        let mapa = match self.tipo {
            Tipo::Titulo => "titulo",
            Tipo::Nivel1 => "nivel1",
            Tipo::Nivel2 => "nivel2",
            Tipo::Nivel3 => "nivel3",
            Tipo::Nivel4 => "nivel4",
        };
        self.escenario
            .cargar(assets.mapas[mapa], glob, assets, banco);
        if self.tipo != Tipo::Titulo {
            self.contar_objetos();
        }
    }

    pub fn dibujar(
        &self,
        glob: &mut Globales,
        assets: &Assets,
        pantalla: &museo::Pantalla,
        materiales: &Materiales,
    ) {
        self.escenario.dibujar(glob, assets, pantalla, materiales);
    }

    /// Game1.cambiarNivel: carga el nivel `indice` (que puede ser este mismo) y lo activa. El
    /// nivel que llama sigue con su actualización después.
    fn cambiar_nivel(&mut self, ctx: &mut Contexto, indice: usize) {
        *ctx.nivel_activo = indice;
        let nivel = if indice == ctx.propio {
            self
        } else {
            ctx.otros[indice].as_mut().expect("nivel")
        };
        nivel.cargar(ctx.glob, ctx.assets, ctx.banco);
    }

    pub fn actualizar(&mut self, control: &Control, ctx: &mut Contexto) {
        match self.tipo {
            Tipo::Titulo => {
                if control.press_start {
                    self.fin = true;
                }
                self.cortina(ctx);
                self.escenario.objeto("start").fade(5);
                self.escenario.objeto("titulo").atenuar_aditivo(1);
                self.escenario.objeto("titulo2").atenuar_aditivo(1);
                self.escenario.actualizar(ctx.glob);
            }
            Tipo::Nivel1 => {
                self.cortina(ctx);
                if !ctx.glob.pausa {
                    self.actualizar_nivel(control, ctx);
                    self.mover_camara(ctx.glob, Vec2::ZERO, vec2(1020.0, 600.0));
                }
            }
            Tipo::Nivel2 => {
                self.cortina(ctx);
                if !ctx.glob.pausa {
                    self.actualizar_nivel(control, ctx);
                    self.mover_camara(ctx.glob, vec2(0.0, -100.0), vec2(0.0, 600.0));
                    self.nivel2();
                }
            }
            Tipo::Nivel3 => {
                self.cortina(ctx);
                if ctx.glob.pausa {
                    return;
                }
                self.actualizar_nivel(control, ctx);
                self.mover_camara(ctx.glob, vec2(0.0, -150.0), vec2(1020.0, 600.0));
                self.nivel3();
            }
            Tipo::Nivel4 => {
                self.cortina(ctx);
                self.mover_camara(ctx.glob, Vec2::ZERO, Vec2::ZERO);
                if ctx.glob.pausa {
                    return;
                }
                self.actualizar_nivel(control, ctx);
                self.nivel4();
            }
        }
    }

    /// Nivel2.actualizar: el héroe da la vuelta por los bordes, el bloque empujable convierte
    /// el ladrillo en suelo sólido y el hunter1 reaparece.
    fn nivel2(&mut self) {
        let e = &mut self.escenario;
        let hero = e.objeto("hero").posicion;
        if hero.x > 640.0 && hero.y > 120.0 {
            e.objeto("hero").posicion = vec2(20.0, 20.0);
        }
        let hero = e.objeto("hero").posicion;
        if hero.x < 0.0 && hero.y > 120.0 {
            e.objeto("hero").posicion = vec2(600.0, 20.0);
        }
        let hero = e.objeto("hero").posicion;
        if hero.x < 0.0 && hero.y < 120.0 {
            e.objeto("hero").posicion = vec2(600.0, 160.0);
        }
        let hero = e.objeto("hero").posicion;
        if hero.x > 640.0 && hero.y < 100.0 {
            e.objeto("hero").posicion = vec2(20.0, 160.0);
        }
        e.objeto("ladrillo1").colision = if e.detectar_colision("ladrillo1", "bloque1") {
            Colision::Total
        } else {
            Colision::Superior
        };
        if !e.objeto("hunter1").activo {
            e.respawn("hunter1", vec2(80.0, 400.0), 2.0);
            let h = e.objeto("hunter1");
            h.estado_seleccionado = 0;
            h.gravedad = true;
            h.inclinacion = 0.0;
            h.lugar = Lugar::Medio;
        }
    }

    /// Nivel3.actualizar: plataformas electrificadas por los hunters y hunters que reaparecen.
    fn nivel3(&mut self) {
        let e = &mut self.escenario;
        for i in 1..=4 {
            let p = format!("plataforma{i}");
            if e.detectar_colision(&p, "hero") && e.objeto(&p).color == AMARILLO {
                self.dano |= reproducir_muerte(e);
            }
        }
        for (hunter, plataforma) in [(2, 1), (3, 2), (4, 3), (5, 4)] {
            electrificar(e, hunter, plataforma);
        }
        let reaparecer = |e: &mut Escenario, nombre: &str, caido: bool, x: f32| {
            if caido || !e.objeto(nombre).activo {
                let h = e.objeto(nombre);
                h.activo = false;
                h.estado_seleccionado = 0;
                h.inclinacion = 0.0;
                h.lugar = Lugar::Medio;
                h.gravedad = true;
                e.respawn(nombre, vec2(x, 0.0), 5.0);
            }
        };
        let p = e.objeto("hunter8").posicion;
        reaparecer(e, "hunter8", p.x < 790.0 && p.y > 539.0, 940.0);
        let p = e.objeto("hunter7").posicion;
        reaparecer(e, "hunter7", p.x > 790.0 && p.y > 539.0, 940.0);
        let p = e.objeto("hunter6").posicion;
        reaparecer(e, "hunter6", p.x > 698.0 && p.y > 539.0, 540.0);
        let p = e.objeto("hunter9").posicion;
        reaparecer(e, "hunter9", p.x > 698.0 && p.y > 539.0, 540.0);
    }

    /// Nivel4.actualizar: la máquina incendia los barriles al recibir un disparo, el cañón apunta
    /// con la mira (que persigue al héroe) y dispara la bola.
    fn nivel4(&mut self) {
        let balas = self.balas;
        let e = &mut self.escenario;
        if !e.objeto("fuego1").activo {
            for i in 1..=balas {
                if e.detectar_colision(&format!("bala{i}"), "maquina") {
                    for f in 1..=6 {
                        e.objeto(&format!("fuego{f}")).activo = true;
                    }
                    e.objeto("barril1").reproducir("explosion2");
                }
            }
        }
        if e.objeto("fuego1").activo && e.objeto("barril1").activo {
            for i in 1..=5 {
                let barril = e.objeto(&format!("barril{i}"));
                barril.colision = Colision::Ninguna;
                // new Random(i): la misma caída en cada partida.
                let mut random = xna::Random::new(i);
                barril.posicion.y += 0.5 * (random.next_double() + 0.5) as f32;
                let y = barril.posicion.y;
                e.objeto(&format!("fuego{}", i + 1)).posicion.y = y;
            }
        }
        if e.objeto("barril1").activo && e.objeto("barril1").posicion.y > 480.0 {
            for i in 1..=5 {
                e.objeto(&format!("barril{i}")).activo = false;
                e.objeto(&format!("fuego{}", i + 1)).activo = false;
            }
        }
        if e.objeto("hunter8").estado_seleccionado != 2 {
            let (canon, mira, hero) = (
                e.objeto("cañon").posicion,
                e.objeto("mira").posicion,
                e.objeto("hero").posicion,
            );
            e.objeto("cañon").inclinacion = angulo(canon, mira);
            // El ángulo se recalcula para Y con la X ya movida.
            let a = angulo(mira, hero) as f64;
            e.objeto("mira").posicion.x += (0.0 - a.cos()) as f32 * 1.5;
            let a = angulo(e.objeto("mira").posicion, hero) as f64;
            e.objeto("mira").posicion.y += (0.0 - a.sin()) as f32 * 1.5;
            if e.detectar_colision("hero", "mira") && !e.objeto("bola").activo {
                e.objeto("bola").reproducir("explosion1");
                let bola = e.objeto("bola").posicion;
                e.objeto("fuego7").posicion = bola;
                e.objeto("bola").activo = true;
                e.objeto("fuego7").activo = true;
            }
            if e.detectar_colision("bola", "hero") && e.objeto("bola").activo {
                self.dano |= reproducir_muerte(e);
            }
        }
        if e.objeto("bola").activo {
            if e.detectar_colision("bola", "mira") && e.objeto("bola").activo {
                e.objeto("fuego7").activo = false;
                e.objeto("bola").activo = false;
            }
            let (bola, mira) = (e.objeto("bola").posicion, e.objeto("mira").posicion);
            let a = angulo(bola, mira) as f64;
            e.objeto("bola").posicion.x += (0.0 - a.cos()) as f32 * 2.0;
            let (bola, mira) = (e.objeto("bola").posicion, e.objeto("mira").posicion);
            let a = angulo(bola, mira) as f64;
            e.objeto("bola").posicion.y += (0.0 - a.sin()) as f32 * 2.0;
            let bola = e.objeto("bola").posicion;
            e.objeto("fuego7").posicion = bola;
        } else {
            e.objeto("bola").posicion = vec2(300.0, 100.0);
            e.objeto("fuego7").posicion = vec2(300.0, 100.0);
        }
        for i in 1..=3 {
            let p = format!("plataforma{i}");
            if e.detectar_colision(&p, "hero") && e.objeto(&p).color == AMARILLO {
                self.dano |= reproducir_muerte(e);
            }
        }
        for i in 1..=3 {
            electrificar(e, i, i);
        }
    }

    /// Nivel.cortina: el fundido a negro ("pixel") al empezar, al morir y al terminar el nivel.
    fn cortina(&mut self, ctx: &mut Contexto) {
        let (dano, fin) = (self.dano, self.fin);
        let pixel = self.escenario.objeto("pixel");
        if !pixel.activo && pixel.color[3] > 250 {
            pixel.activo = true;
        }
        if pixel.activo && !dano && !fin {
            pixel.fade_out(3);
        }
        if !pixel.activo && (dano || fin) {
            pixel.activo = true;
        }
        if pixel.activo && (dano || fin) {
            pixel.fade_in(3);
        }
        if self.dano {
            ctx.glob.pausa = true;
            ctx.glob.activar_shaders(Shader::Negative);
        } else if !self.fin {
            ctx.glob.desactivar_shaders();
        }
        if self.fin && *ctx.nivel_activo > 0 {
            self.escenario.objeto("hero").activo = false;
        }
        if self.fin && self.escenario.objeto("pixel").color[3] == 255 {
            let siguiente = (*ctx.nivel_activo + 1) % 5;
            self.cambiar_nivel(ctx, siguiente);
            self.fin = false;
            ctx.glob.pausa = false;
        }
        if self.dano && self.escenario.objeto("pixel").color[3] == 255 {
            let actual = *ctx.nivel_activo;
            self.cambiar_nivel(ctx, actual);
            self.dano = false;
            ctx.glob.pausa = false;
        }
    }

    /// Nivel.contarObjetos: cuántos objetos de cada clase hay (por nombre).
    fn contar_objetos(&mut self) {
        let contar = |parte: &str| {
            self.escenario
                .objetos
                .iter()
                .filter(|o| o.nombre.contains(parte))
                .count() as i32
        };
        self.huevos = contar("huevo");
        self.paredes = contar("pared");
        self.fachadas = contar("fachada");
        self.balas = contar("bala");
        self.pilas = contar("pila");
        self.bloques = contar("bloque");
        self.mapas = contar("mapa");
        self.contador_mapas = self.mapas;
        self.hunters = contar("hunter");
        self.kamikazes = contar("kamikaze");
        self.pajaros = contar("pajaro");
        self.bateria = 0;
        self.saltando = false;
        self.escenario.objeto("barra").estados[0].cuadro.width = self.bateria;
    }

    /// Nivel.moverCamara: centra la cámara en el héroe dentro de [min, max - pantalla].
    fn mover_camara(&mut self, glob: &mut Globales, min: Vec2, max: Vec2) {
        let hero = self.escenario.objeto("hero");
        let p = &mut glob.pantalla;
        p.posicion.x = hero.posicion.x + (hero.cuerpo.width / 2) as f32 - (p.ancho / 2) as f32;
        p.posicion.x = clamp(p.posicion.x, min.x, max.x - p.ancho as f32);
        p.posicion.y = hero.posicion.y + (hero.cuerpo.height / 2) as f32 - (p.alto / 2) as f32;
        p.posicion.y = clamp(p.posicion.y, min.y, max.y - p.alto as f32);
        p.reset();
    }

    /// Nivel.actualizarNivel.
    fn actualizar_nivel(&mut self, control: &Control, ctx: &mut Contexto) {
        if *ctx.nivel_activo != 0 && control.reiniciar {
            let actual = *ctx.nivel_activo;
            self.cambiar_nivel(ctx, actual);
        }
        if self.escenario.objeto("hero").activo {
            self.mover_personaje(control);
        }
        self.actualizar_objetos(ctx.glob);
        self.escenario.actualizar(ctx.glob);
    }

    /// Nivel.actualizarObjetos: balas, mapas, pilas, enemigos y el vórtice de salida.
    fn actualizar_objetos(&mut self, glob: &mut Globales) {
        let e = &mut self.escenario;
        for i in 1..=self.fachadas {
            let f = format!("fachada{i}");
            if e.detectar_colision("hero", &f) {
                e.objeto(&f).fade_out(4);
            } else {
                e.objeto(&f).fade_in(4);
            }
        }
        for i in 1..=self.balas {
            let b = format!("bala{i}");
            if !e.objeto(&b).activo {
                continue;
            }
            e.objeto(&b).velocidad += SEGUNDOS_POR_PASO * 2.0;
            if e.objeto(&b).velocidad > 10.0 {
                e.objeto(&b).activo = false;
            }
            for j in 1..=self.huevos {
                let h = format!("huevo{j}");
                if e.detectar_colision(&b, &h)
                    && e.objeto(&h).activo
                    && e.objeto(&h).estado_seleccionado == 0
                {
                    e.objeto(&b).activo = false;
                    e.objeto(&h).cambiar_estado(1);
                }
            }
            for j in 1..=self.hunters {
                let h = format!("hunter{j}");
                if e.detectar_colision(&b, &h) && e.objeto(&h).activo {
                    e.objeto(&b).activo = false;
                    e.objeto(&h).cambiar_estado(2);
                    e.objeto(&h).reproducir("hunter dead");
                }
            }
            for j in 1..=self.paredes {
                let p = format!("pared{j}");
                if e.detectar_colision(&b, &p) && e.objeto(&p).activo {
                    e.objeto(&b).activo = false;
                }
            }
            for j in 1..=self.kamikazes {
                let k = format!("kamikaze{j}");
                if e.detectar_colision(&b, &k) && e.objeto(&k).activo {
                    e.objeto(&b).activo = false;
                    e.objeto(&k).cambiar_estado(2);
                    e.objeto(&k).reproducir("kamikaze dead");
                }
            }
            let bala = e.objeto(&b);
            if bala.efectos == Efectos::NINGUNO {
                bala.posicion.x += bala.velocidad;
            } else if bala.efectos == Efectos::HORIZONTAL {
                bala.posicion.x -= bala.velocidad;
            }
        }
        if e.detectar_colision("hero", "vortice") && e.objeto("vortice").activo {
            self.fin = true;
        }
        if self.contador_mapas == 0 {
            for i in 1..=self.hunters {
                e.objeto(&format!("hunter{i}")).activo = false;
            }
            for i in 1..=self.kamikazes {
                e.objeto(&format!("kamikaze{i}")).activo = false;
            }
            if !e.objeto("vortice").activo {
                e.objeto("vortice").reproducir("vortice");
            }
            e.objeto("vortice").activo = true;
            glob.activar_shaders(Shader::Brightness);
            if let Some(post) = &mut glob.post {
                post.intensidad = 2.0;
            }
        } else if !self.dano {
            glob.desactivar_shaders();
        }
        for i in 1..=self.mapas {
            let m = format!("mapa{i}");
            if e.detectar_colision("hero", &m)
                && e.objeto(&m).activo
                && !e.objeto(&m).fuera_de_camara
            {
                let mapa = e.objeto(&m);
                mapa.reproducir("paper");
                mapa.posicion = vec2((80 + 40 * i) as f32, 10.0);
                mapa.fuera_de_camara = true;
                mapa.no_shader_camara = true;
                mapa.lugar = Lugar::Frente;
                self.contador_mapas -= 1;
            }
        }
        for i in 1..=self.pilas {
            let p = format!("pila{i}");
            if e.detectar_colision("hero", &p) && e.objeto(&p).activo {
                self.bateria = 100;
                e.objeto("barra").estados[0].cuadro.width = self.bateria;
                let pila = e.objeto(&p);
                pila.reproducir("charge");
                if pila.sonidos[0].instancia.borrow().sonando() {
                    pila.activo = false;
                }
            }
        }
        for i in 1..=self.bloques {
            let b = format!("bloque{i}");
            if e.objeto(&b).activo && e.detectar_colision("hero", &b) {
                let (izquierda, derecha, v) = {
                    let hero = e.objeto("hero");
                    (
                        hero.colision_izquierda,
                        hero.colision_derecha,
                        hero.velocidad,
                    )
                };
                // Mira siempre las colisiones de "bloque1", sea cual sea el bloque.
                if izquierda && !e.objeto("bloque1").colision_izquierda {
                    e.objeto(&b).posicion.x -= v / 2.0;
                }
                if derecha && !e.objeto("bloque1").colision_derecha {
                    e.objeto(&b).posicion.x += v / 2.0;
                }
            }
        }
        for i in 1..=self.pajaros {
            let p = format!("pajaro{i}");
            if e.objeto(&p).activo {
                let hero = e.objeto("hero").cuerpo;
                if e.objeto(&p).cuerpo.intersects(&hero) {
                    let hero = e.objeto("hero").posicion;
                    let pajaro = e.objeto(&p);
                    pajaro.estado_seleccionado = 1;
                    pajaro.movimento = true;
                    let emisor = pajaro.posicion;
                    pajaro.reproducir_desde("crow", emisor, hero);
                }
                let pajaro = e.objeto(&p);
                if pajaro.posicion.x > 1024.0 || pajaro.cuerpo.right() < 0 {
                    pajaro.activo = false;
                }
            }
        }
        let v_alto = glob.pantalla.v_alto as f32;
        let pantalla_y = glob.pantalla.posicion.y;
        for i in 1..=self.hunters {
            let h = format!("hunter{i}");
            if e.objeto(&h).estado_seleccionado == 2 {
                caer_muerto(e.objeto(&h), v_alto + pantalla_y);
                continue;
            }
            // Color_.B -= (byte)(10 * (TotalSeconds * 10)): el azul va bajando y, entre 150 y
            // 100, el hunter está electrificado.
            let hunter = e.objeto(&h);
            hunter.color[2] =
                hunter.color[2].wrapping_sub((10.0 * (SEGUNDOS_POR_PASO * 10.0)) as u8);
            hunter.estado_seleccionado = if hunter.color[2] > 100 && hunter.color[2] < 150 {
                1
            } else {
                0
            };
            if e.detectar_colision("hero", &h) {
                let i_h = e.indice(&h);
                if pisar(e, &mut self.saltando, &mut self.distancia, i_h)
                    && e.objeto(&h).estado_seleccionado == 0
                    && e.objeto(&h).activo
                {
                    e.objeto(&h).cambiar_estado(2);
                    e.objeto(&h).reproducir("hunter dead");
                } else if e.objeto(&h).activo {
                    self.dano = true;
                    e.objeto("hero").reproducir("dead");
                }
            }
            let hunter = e.objeto(&h);
            hunter.efectos = Efectos::horizontal_si(hunter.ida_x);
            let hero = e.objeto("hero").posicion;
            let hunter = e.objeto(&h);
            let emisor = hunter.posicion;
            if hunter.movimento && hunter.activo && hunter.estado_seleccionado == 0 {
                hunter.reproducir_desde("hunter move", emisor, hero);
            } else if hunter.activo && hunter.estado_seleccionado == 1 {
                hunter.reproducir_desde("electricity", emisor, hero);
            }
        }
        for i in 1..=self.kamikazes {
            let k = format!("kamikaze{i}");
            if e.objeto(&k).estado_seleccionado == 2 {
                caer_muerto(e.objeto(&k), v_alto + pantalla_y);
                continue;
            }
            if e.detectar_colision("hero", &k) {
                let i_k = e.indice(&k);
                if pisar(e, &mut self.saltando, &mut self.distancia, i_k)
                    && e.objeto(&k).estado_seleccionado == 0
                    && e.objeto(&k).activo
                {
                    e.objeto(&k).cambiar_estado(2);
                    e.objeto(&k).reproducir("kamikaze dead");
                } else if e.objeto(&k).activo && !self.dano {
                    self.dano = true;
                    e.objeto("hero").reproducir("dead");
                }
            }
            let hero = e.objeto("hero").posicion;
            let kamikaze = e.objeto(&k);
            let emisor = kamikaze.posicion;
            if kamikaze.estado_seleccionado == 0 {
                kamikaze.reproducir_desde("kamikaze move", emisor, hero);
            } else if kamikaze.estado_seleccionado == 1 {
                kamikaze.reproducir_desde("electricity", emisor, hero);
            }
            if kamikaze.ida_y && kamikaze.estado_seleccionado != 2 {
                kamikaze.efectos = Efectos::VERTICAL;
                kamikaze.estado_seleccionado = 0;
            } else if kamikaze.estado_seleccionado != 2 {
                kamikaze.efectos = Efectos::NINGUNO;
                kamikaze.estado_seleccionado = 1;
            }
        }
    }

    /// Nivel.moverPersonaje: Z dispara (si hay batería), X salta y las flechas mueven.
    fn mover_personaje(&mut self, control: &Control) {
        let e = &mut self.escenario;
        if control.press_z && self.bateria > 0 {
            e.objeto("hero").reproducir("shot");
            let b = format!("bala{}", self.bateria / 10);
            let (efectos, cuerpo, y) = {
                let hero = e.objeto("hero");
                (hero.efectos, hero.cuerpo, hero.posicion.y)
            };
            let bala = e.objeto(&b);
            bala.efectos = efectos;
            if efectos == Efectos::NINGUNO {
                bala.posicion = vec2(cuerpo.right() as f32, y + 17.0);
            } else if efectos == Efectos::HORIZONTAL {
                bala.posicion = vec2(cuerpo.left() as f32, y + 17.0);
            }
            bala.velocidad = 9.0;
            bala.activo = true;
            self.bateria -= 10;
            e.objeto("barra").estados[0].cuadro.width = self.bateria;
        }
        let hero = e.objeto("hero");
        if control.hold_right {
            hero.velocidad = 4.0;
            hero.posicion.x += hero.velocidad;
            if !self.saltando && hero.estado_seleccionado != 1 {
                hero.cambiar_estado(1);
            }
            hero.efectos = Efectos::NINGUNO;
            hero.reproducir("walk");
        }
        if control.hold_left {
            hero.velocidad = 4.0;
            hero.posicion.x -= hero.velocidad;
            if !self.saltando && hero.estado_seleccionado != 1 {
                hero.cambiar_estado(1);
            }
            hero.efectos = Efectos::HORIZONTAL;
            hero.reproducir("walk");
        }
        if !control.hold_left && !control.hold_right {
            if !self.saltando && hero.estado_seleccionado != 0 {
                hero.cambiar_estado(0);
            }
            hero.stop("walk");
        }
        if !self.saltando && control.press_x && hero.colision_superior {
            self.distancia = (hero.posicion.y - 50.0) as i32;
            hero.fuerza_gravedad = -6.0;
            self.saltando = true;
        }
        if self.saltando {
            hero.reproducir("jump");
            if hero.posicion.y > self.distancia as f32 {
                hero.posicion.y -= 1.9;
                if hero.estado_seleccionado != 2 {
                    hero.cambiar_estado(2);
                }
            } else {
                self.saltando = false;
            }
        }
        if !hero.colision_superior {
            hero.stop("walk");
            if hero.estado_seleccionado != 2 {
                hero.cambiar_estado(2);
            }
        }
        if hero.colision_inferior {
            self.saltando = false;
        }
    }
}

/// Nivel.pisar: el héroe cae sobre el enemigo y rebota.
fn pisar(e: &mut Escenario, saltando: &mut bool, distancia: &mut i32, enemigo: usize) -> bool {
    let (cuerpo_enemigo, activo) = (e.objetos[enemigo].cuerpo, e.objetos[enemigo].activo);
    let hero = e.objeto("hero");
    if hero.cuerpo.bottom() >= cuerpo_enemigo.top()
        && hero.cuerpo.bottom() <= cuerpo_enemigo.top() + 10
        && !*saltando
        && activo
    {
        *distancia = (hero.posicion.y - 70.0) as i32;
        hero.fuerza_gravedad = -8.0;
        *saltando = true;
        return true;
    }
    false
}

/// El enemigo muerto cae por delante de todo hasta salir de la pantalla.
fn caer_muerto(enemigo: &mut crate::motor::objeto::Objeto, borde: f32) {
    if enemigo.posicion.y <= borde {
        enemigo.lugar = Lugar::Frente;
        enemigo.gravedad = false;
        enemigo.posicion.y += 4.0;
    } else {
        enemigo.activo = false;
    }
}

/// Nivel3/Nivel4: la plataforma se pone amarilla (electrificada) mientras su hunter lo está.
fn electrificar(e: &mut Escenario, hunter: i32, plataforma: i32) {
    let h = e.objeto(&format!("hunter{hunter}"));
    let electrico = h.activo && h.estado_seleccionado == 1;
    e.objeto(&format!("plataforma{plataforma}")).color = if electrico { AMARILLO } else { GRIS };
}

/// hero.reproducir("dead") y daño si su cuarto sonido (Sonidos[3], "dead") ya está sonando.
fn reproducir_muerte(e: &mut Escenario) -> bool {
    let hero = e.objeto("hero");
    hero.reproducir("dead");
    hero.sonidos[3].instancia.borrow().sonando()
}
