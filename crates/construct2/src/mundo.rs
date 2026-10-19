// El mundo del runtime: las instancias del layout actual (Sprites y Texts), el tiempo de juego,
// los ticks de los comportamientos (con las colisiones contra sólidos de Bullet), la creación y
// destrucción de instancias, el cambio de layout y el dibujo por capas.

use std::collections::HashMap;
use std::f64::consts::PI;

use macroquad::math::DVec2;
use macroquad::prelude::*;

use crate::comportamientos::{Bala, Comportamiento, Envolver, Torreta};
use crate::datos::{Clase, Instancia, Proyecto, Tipo};
use crate::fisica::MundoFisico;
use crate::geometria::{
    a_radianes, angulo_acotado, angulo_hacia, diferencia_angular, distancia, en_sentido_horario,
    piso_c2, rotar_angulo,
};
use crate::particulas::Particulas;
use crate::sprite::Sprite;
use crate::texto::{Fuentes, Texto};
use crate::tiempo::Kahan;

pub enum Inst {
    Sprite(Sprite),
    Texto(Texto),
    Particulas(Particulas),
}

impl Inst {
    pub fn tipo(&self) -> usize {
        match self {
            Inst::Sprite(s) => s.tipo,
            Inst::Texto(t) => t.tipo,
            Inst::Particulas(p) => p.tipo,
        }
    }

    pub fn uid(&self) -> u32 {
        match self {
            Inst::Sprite(s) => s.uid,
            Inst::Texto(t) => t.uid,
            Inst::Particulas(p) => p.uid,
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            Inst::Sprite(s) => s.visible,
            Inst::Texto(t) => t.visible,
            Inst::Particulas(p) => p.visible,
        }
    }

    pub fn destruido(&self) -> bool {
        match self {
            Inst::Sprite(s) => s.destruido,
            Inst::Texto(t) => t.destruido,
            Inst::Particulas(p) => p.destruido,
        }
    }

    pub fn posicion(&self) -> (f64, f64) {
        match self {
            Inst::Sprite(s) => (s.x, s.y),
            Inst::Texto(t) => (t.x, t.y),
            Inst::Particulas(p) => (p.x, p.y),
        }
    }

    pub fn capa(&self) -> usize {
        match self {
            Inst::Sprite(s) => s.capa,
            Inst::Texto(t) => t.capa,
            Inst::Particulas(p) => p.capa,
        }
    }
}

pub struct Mundo {
    pub proyecto: Proyecto,
    texturas: HashMap<String, Texture2D>,
    fuentes: Fuentes,
    pub layout: usize,
    /// En orden de creación: las del layout por capas y en su orden Z, y las creadas después al
    /// final (encima de su capa).
    pub instancias: Vec<Inst>,
    siguiente_uid: u32,
    tiempo: Kahan,
    /// dt del tick actual (ya multiplicado por la escala de tiempo).
    pub dt: f64,
    /// System.SetTimescale.
    pub escala_tiempo: f64,
    /// Ticks completados (tickcount).
    pub ticks: u64,
    /// Colisiones que registró un comportamiento en este tick (registerCollision), en uid.
    registradas: Vec<(u32, u32)>,
    /// Objetivos de Turret: la lista es del tipo de la torreta y guarda tipos de objeto.
    objetivos: HashMap<usize, Vec<usize>>,
    iniciado: bool,
    /// El mundo de Box2D del comportamiento Physics (se crea con el primer cuerpo).
    pub(crate) fisica: Option<MundoFisico>,
    /// Scroll del layout: el centro de la vista.
    pub scroll_x: f64,
    pub scroll_y: f64,
    /// Visibilidad de las capas (System.SetLayerVisible).
    capas_visibles: Vec<bool>,
}

impl Mundo {
    pub fn nuevo(
        proyecto: Proyecto,
        texturas: HashMap<String, Texture2D>,
        fuentes: Fuentes,
    ) -> Self {
        Self {
            proyecto,
            texturas,
            fuentes,
            layout: 0,
            instancias: Vec::new(),
            siguiente_uid: 0,
            tiempo: Kahan::default(),
            dt: 0.0,
            escala_tiempo: 1.0,
            ticks: 0,
            registradas: Vec::new(),
            objetivos: HashMap::new(),
            iniciado: false,
            fisica: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
            capas_visibles: Vec::new(),
        }
    }

    // --- tiempo y layouts --------------------------------------------------------------------

    /// Tiempo de juego (kahanTime): solo avanza con la escala de tiempo.
    pub fn tiempo(&self) -> f64 {
        self.tiempo.suma
    }

    /// Principio del tick: dt × escala de tiempo, sumado al tiempo de juego.
    pub fn avanzar(&mut self, dt_real: f64) {
        self.dt = dt_real * self.escala_tiempo;
        self.tiempo.sumar(self.dt);
    }

    pub fn nombre_layout(&self) -> &str {
        &self.proyecto.layouts[self.layout].nombre
    }

    pub fn en_layout(&self, nombre: &str) -> bool {
        self.nombre_layout().eq_ignore_ascii_case(nombre)
    }

    fn nuevo_uid(&mut self) -> u32 {
        self.siguiente_uid += 1;
        self.siguiente_uid
    }

    /// Arranque o cambio de layout: se destruyen las instancias y se crean las del nuevo (cada
    /// una con un uid nuevo), y se olvidan las colisiones registradas. Devuelve si era el mismo
    /// layout: entonces el runtime también borra las esperas de System.Wait. Quien llama lanza
    /// "On start of layout".
    pub fn cambiar_layout(&mut self, nombre: &str) -> bool {
        let indice = self
            .proyecto
            .layouts
            .iter()
            .position(|l| l.nombre.eq_ignore_ascii_case(nombre))
            .expect("layout");
        let mismo = self.iniciado && indice == self.layout;
        self.iniciado = true;
        // Fin del layout: se destruyen todas las instancias (los cuerpos de Physics también).
        for i in 0..self.instancias.len() {
            self.destruir(i);
        }
        self.quitar_destruidas();
        self.layout = indice;
        self.registradas.clear();
        // El scroll empieza en el centro de la ventana.
        self.scroll_x = self.proyecto.ancho / 2.0;
        self.scroll_y = self.proyecto.alto / 2.0;
        self.capas_visibles = self.proyecto.layouts[indice]
            .capas
            .iter()
            .map(|c| c.visible)
            .collect();
        for c in 0..self.proyecto.layouts[indice].capas.len() {
            for k in 0..self.proyecto.layouts[indice].capas[c].instancias.len() {
                let uid = self.nuevo_uid();
                let inst = &self.proyecto.layouts[indice].capas[c].instancias[k];
                let nueva = self.nueva_instancia(inst, c, uid, inst.x, inst.y);
                self.instancias.push(nueva);
                self.crear_cuerpo(self.instancias.len() - 1);
            }
        }
        mismo
    }

    /// createInstanceFromInit según el plugin del tipo.
    fn nueva_instancia(&self, inst: &Instancia, capa: usize, uid: u32, x: f64, y: f64) -> Inst {
        let tipo = &self.proyecto.tipos[inst.tipo];
        if !tipo.animaciones.is_empty() {
            Inst::Sprite(Sprite::nuevo(inst, tipo, capa, uid, x, y))
        } else if let Some(textura) = &tipo.textura {
            Inst::Particulas(Particulas::nueva(inst, textura, capa, uid, x, y))
        } else {
            Inst::Texto(Texto::nuevo(inst, capa, uid, x, y))
        }
    }

    // --- instancias --------------------------------------------------------------------------

    /// Las instancias de un tipo, en orden de creación (las destruidas siguen hasta que se
    /// quitan, como en las listas del runtime).
    pub fn de(&self, tipo: usize) -> Vec<usize> {
        (0..self.instancias.len())
            .filter(|&i| self.instancias[i].tipo() == tipo)
            .collect()
    }

    pub fn primera(&self, tipo: usize) -> Option<usize> {
        (0..self.instancias.len()).find(|&i| self.instancias[i].tipo() == tipo)
    }

    pub fn uid(&self, i: usize) -> u32 {
        self.instancias[i].uid()
    }

    pub fn por_uid(&self, uid: u32) -> Option<usize> {
        self.instancias.iter().position(|i| i.uid() == uid)
    }

    pub fn tipo(&self, i: usize) -> &Tipo {
        &self.proyecto.tipos[self.instancias[i].tipo()]
    }

    pub fn sprite(&self, i: usize) -> &Sprite {
        match &self.instancias[i] {
            Inst::Sprite(s) => s,
            _ => unreachable!("no es un Sprite"),
        }
    }

    pub fn sprite_mut(&mut self, i: usize) -> &mut Sprite {
        match &mut self.instancias[i] {
            Inst::Sprite(s) => s,
            _ => unreachable!("no es un Sprite"),
        }
    }

    pub fn texto(&self, i: usize) -> &Texto {
        match &self.instancias[i] {
            Inst::Texto(t) => t,
            _ => unreachable!("no es un Text"),
        }
    }

    pub fn texto_mut(&mut self, i: usize) -> &mut Texto {
        match &mut self.instancias[i] {
            Inst::Texto(t) => t,
            _ => unreachable!("no es un Text"),
        }
    }

    pub fn particulas_mut(&mut self, i: usize) -> &mut Particulas {
        match &mut self.instancias[i] {
            Inst::Particulas(p) => p,
            _ => unreachable!("no es un Particles"),
        }
    }

    pub fn posicion(&self, i: usize) -> (f64, f64) {
        self.instancias[i].posicion()
    }

    /// SetPosition (Sprite o Text).
    pub fn set_posicion(&mut self, i: usize, x: f64, y: f64) {
        match &mut self.instancias[i] {
            Inst::Sprite(s) => (s.x, s.y) = (x, y),
            Inst::Texto(t) => (t.x, t.y) = (x, y),
            Inst::Particulas(p) => (p.x, p.y) = (x, y),
        }
    }

    pub fn visible(&self, i: usize) -> bool {
        self.instancias[i].visible()
    }

    pub fn set_visible(&mut self, i: usize, visible: bool) {
        match &mut self.instancias[i] {
            Inst::Sprite(s) => s.visible = visible,
            Inst::Texto(t) => t.visible = visible,
            Inst::Particulas(p) => p.visible = visible,
        }
    }

    /// SetVisible sobre todas las instancias de un tipo.
    pub fn set_visible_tipo(&mut self, tipo: usize, visible: bool) {
        for i in self.de(tipo) {
            self.set_visible(i, visible);
        }
    }

    /// Destroy: la instancia sigue en las listas hasta que se quitan las destruidas
    /// (ClearDeathRow).
    pub fn destruir(&mut self, i: usize) {
        match &mut self.instancias[i] {
            Inst::Sprite(s) => s.destruido = true,
            Inst::Texto(t) => t.destruido = true,
            Inst::Particulas(p) => p.destruido = true,
        }
    }

    pub fn destruir_tipo(&mut self, tipo: usize) {
        for i in self.de(tipo) {
            self.destruir(i);
        }
    }

    /// ClearDeathRow. Las destruidas se procesan por tipos (en el orden en que aparece cada tipo)
    /// y, dentro de cada tipo, en orden; los cuerpos de Physics se destruyen entonces.
    pub fn quitar_destruidas(&mut self) {
        if self.fisica.is_some() {
            let mut tipos: Vec<usize> = Vec::new();
            for i in self.por_capas() {
                let inst = &self.instancias[i];
                if inst.destruido() && !tipos.contains(&inst.tipo()) {
                    tipos.push(inst.tipo());
                }
            }
            let capas = self.por_capas();
            let mut orden: Vec<usize> = Vec::new();
            for &t in &tipos {
                for &i in &capas {
                    let inst = &self.instancias[i];
                    if inst.destruido() && inst.tipo() == t {
                        orden.push(i);
                    }
                }
            }
            for (k, &i) in orden.iter().enumerate() {
                let vivas: Vec<usize> = (0..self.instancias.len())
                    .filter(|j| !orden[..k].contains(j))
                    .collect();
                self.al_destruir_fisica(i, &vivas);
            }
        }
        self.instancias.retain(|i| !i.destruido());
    }

    /// Las instancias por capas y, dentro de cada capa, en su orden Z.
    fn por_capas(&self) -> Vec<usize> {
        let n = self.proyecto.layouts[self.layout].capas.len();
        (0..n)
            .flat_map(|c| {
                (0..self.instancias.len()).filter(move |&i| self.instancias[i].capa() == c)
            })
            .collect()
    }

    /// System.CreateObject: con las propiedades de la primera instancia del tipo en el proyecto,
    /// en la capa indicada y encima de las demás.
    pub fn crear(&mut self, tipo: usize, capa: usize, x: f64, y: f64) -> usize {
        let uid = self.nuevo_uid();
        let modelo = self.proyecto.instancia_por_defecto(tipo);
        let inst = self.nueva_instancia(modelo, capa, uid, x, y);
        self.instancias.push(inst);
        let i = self.instancias.len() - 1;
        self.crear_cuerpo(i);
        i
    }

    // --- animaciones -------------------------------------------------------------------------

    pub fn set_anim(&mut self, i: usize, nombre: &str, desde: i32) {
        let Self {
            proyecto,
            instancias,
            ..
        } = self;
        if let Inst::Sprite(s) = &mut instancias[i] {
            s.set_anim(&proyecto.tipos[s.tipo], nombre, desde);
        }
    }

    pub fn anim_es(&self, i: usize, nombre: &str) -> bool {
        let s = self.sprite(i);
        s.anim_es(&self.proyecto.tipos[s.tipo], nombre)
    }

    pub fn set_escala(&mut self, i: usize, escala: f64) {
        let Self {
            proyecto,
            instancias,
            ..
        } = self;
        if let Inst::Sprite(s) = &mut instancias[i] {
            s.set_escala(&proyecto.tipos[s.tipo], escala);
        }
    }

    /// Tick de la animación de un Sprite: devuelve el nombre de la animación que terminó.
    pub fn tick_animacion(&mut self, i: usize) -> Option<String> {
        let dt = self.dt;
        let Self {
            proyecto,
            instancias,
            ..
        } = self;
        match &mut instancias[i] {
            Inst::Sprite(s) if !s.destruido => s.tick(&proyecto.tipos[s.tipo], dt),
            Inst::Particulas(p) if !p.destruido => {
                if p.tick(dt) {
                    p.destruido = true;
                }
                None
            }
            _ => None,
        }
    }

    // --- colisiones --------------------------------------------------------------------------

    /// runtime.testOverlap (solo entre Sprites).
    pub fn solapa(&self, a: usize, b: usize) -> bool {
        if a == b {
            return false;
        }
        match (&self.instancias[a], &self.instancias[b]) {
            (Inst::Sprite(sa), Inst::Sprite(sb)) => sa.solapa(
                &self.proyecto.tipos[sa.tipo],
                sb,
                &self.proyecto.tipos[sb.tipo],
            ),
            _ => false,
        }
    }

    fn solapa_con(&self, a: usize, b: Option<usize>) -> bool {
        b.is_some_and(|b| self.solapa(a, b))
    }

    /// IsOverlapping: las instancias de `candidatas` que tocan alguna del tipo `otro`.
    pub fn solapan(&self, candidatas: &[usize], otro: usize) -> Vec<usize> {
        let otros = self.de(otro);
        candidatas
            .iter()
            .copied()
            .filter(|&i| otros.iter().any(|&j| self.solapa(i, j)))
            .collect()
    }

    /// Condición invertida "not IsOverlapping": las que no tocan ninguna.
    pub fn no_solapan(&self, candidatas: &[usize], otro: usize) -> Vec<usize> {
        let si = self.solapan(candidatas, otro);
        candidatas
            .iter()
            .copied()
            .filter(|i| !si.contains(i))
            .collect()
    }

    /// contains_pt de una instancia (los Text, solo su caja).
    /// contains_pt con un punto de la pantalla (se pasa a las coordenadas de la capa de la
    /// instancia, con su scroll y parallax).
    pub fn contiene(&self, i: usize, p: DVec2) -> bool {
        let (izq, arr) = self.vista_capa(self.instancias[i].capa());
        let (x, y) = (p.x + izq, p.y + arr);
        match &self.instancias[i] {
            Inst::Sprite(s) => s.contiene(&self.proyecto.tipos[s.tipo], x, y),
            Inst::Texto(t) => t.caja().contains(vec2(x as f32, y as f32)),
            Inst::Particulas(_) => false,
        }
    }

    /// La esquina superior izquierda de la vista de una capa (canvasToLayer del punto 0, 0): el
    /// scroll multiplicado por el parallax de la capa.
    pub fn vista_capa(&self, capa: usize) -> (f64, f64) {
        let parallax = self.proyecto.layouts[self.layout].capas[capa].parallax;
        let (ox, oy) = (self.proyecto.ancho / 2.0, self.proyecto.alto / 2.0);
        let par_x = (self.scroll_x - ox) * parallax.x + ox;
        let par_y = (self.scroll_y - oy) * parallax.y + oy;
        (par_x - ox, par_y - oy)
    }

    /// System.SetLayerVisible.
    pub fn set_capa_visible(&mut self, capa: usize, visible: bool) {
        if let Some(v) = self.capas_visibles.get_mut(capa) {
            *v = visible;
        }
    }

    /// Layout.scrollToX/Y: sin «Unbounded scrolling», la vista no sale del layout.
    pub fn desplazar_a(&mut self, x: f64, y: f64) {
        let layout = &self.proyecto.layouts[self.layout];
        let (mut x, mut y) = (x, y);
        if !layout.ilimitado {
            let (ax, ay) = (self.proyecto.ancho / 2.0, self.proyecto.alto / 2.0);
            if x > layout.ancho - ax {
                x = layout.ancho - ax;
            }
            if x < ax {
                x = ax;
            }
            if y > layout.alto - ay {
                y = layout.alto - ay;
            }
            if y < ay {
                y = ay;
            }
        }
        self.scroll_x = x;
        self.scroll_y = y;
    }

    /// tick2 de los comportamientos (después de los eventos): ScrollTo lleva la vista a la media
    /// de sus instancias.
    pub fn tick2(&mut self) {
        let (mut sx, mut sy, mut n) = (0.0, 0.0, 0);
        for inst in &self.instancias {
            if let Inst::Sprite(s) = inst
                && !s.destruido
                && s.comportamientos
                    .iter()
                    .any(|c| matches!(c, Comportamiento::Desplazar(d) if d.habilitado))
            {
                sx += s.x;
                sy += s.y;
                n += 1;
            }
        }
        if n > 0 {
            self.desplazar_a(sx / n as f64, sy / n as f64);
        }
    }

    /// Las instancias de un tipo bajo un punto (Touch "On touched object", Mouse "Is over
    /// object"): no importa si son visibles.
    pub fn en_punto(&self, tipo: usize, p: DVec2) -> Vec<usize> {
        self.de(tipo)
            .into_iter()
            .filter(|&i| self.contiene(i, p))
            .collect()
    }

    /// testOverlapSolid: el primer sólido habilitado que toca (por tipos, en orden).
    pub fn solido_que_solapa(&self, i: usize) -> Option<usize> {
        for (tipo, t) in self.proyecto.tipos.iter().enumerate() {
            if !t.tiene(Clase::Solido) {
                continue;
            }
            for j in self.de(tipo) {
                let habilitado = matches!(&self.instancias[j],
                    Inst::Sprite(s) if s.solido().is_some_and(|s| s.habilitado));
                if habilitado && self.solapa(i, j) {
                    return Some(j);
                }
            }
        }
        None
    }

    /// registerCollision: un comportamiento empujó a `a` fuera de `b`; "On collision" lo cuenta
    /// aunque ya no se solapen.
    pub fn registrar_colision_uid(&mut self, a: u32, b: u32) {
        self.registradas.push((a, b));
    }

    pub fn registrar_colision(&mut self, a: usize, b: usize) {
        self.registradas.push((self.uid(a), self.uid(b)));
    }

    pub fn colision_registrada(&self, a: usize, b: usize) -> bool {
        let (ua, ub) = (self.uid(a), self.uid(b));
        self.registradas
            .iter()
            .any(|&(x, y)| (x == ua && y == ub) || (x == ub && y == ua))
    }

    /// Después de la hoja de eventos se olvidan las colisiones registradas.
    pub fn olvidar_colisiones(&mut self) {
        self.registradas.clear();
    }

    // --- comportamientos ---------------------------------------------------------------------

    /// Turret.AddTarget: la lista es del tipo de la torreta.
    pub fn agregar_objetivo(&mut self, tipo_torreta: usize, tipo_objetivo: usize) {
        let lista = self.objetivos.entry(tipo_torreta).or_default();
        if !lista.contains(&tipo_objetivo) {
            lista.push(tipo_objetivo);
        }
    }

    /// Ticks de los comportamientos: por tipo de objeto (en el orden del proyecto), cada
    /// instancia del tipo y sus comportamientos en orden; después los posticks (Platform).
    pub fn tick_comportamientos(&mut self) {
        let mut orden: Vec<usize> = (0..self.instancias.len())
            .filter(|&i| matches!(&self.instancias[i], Inst::Sprite(s) if !s.destruido))
            .collect();
        orden.sort_by_key(|&i| self.instancias[i].tipo());
        let dt = self.dt;
        for &i in &orden {
            for k in 0..self.sprite(i).comportamientos.len() {
                // Physics trabaja sobre la instancia (puede rehacer su cuerpo).
                if matches!(self.sprite(i).comportamientos[k], Comportamiento::Fisica(_)) {
                    self.tick_fisica(i);
                    continue;
                }
                let mut c = std::mem::replace(
                    &mut self.sprite_mut(i).comportamientos[k],
                    Comportamiento::Otro,
                );
                match &mut c {
                    Comportamiento::Bala(b) => self.tick_bala(i, b),
                    Comportamiento::Torreta(t) => self.tick_torreta(i, t),
                    Comportamiento::Movimiento(m) => {
                        let s = self.sprite_mut(i);
                        m.tick(&mut s.x, &mut s.y, dt);
                    }
                    Comportamiento::Envolver(e) => self.tick_envolver(i, e),
                    _ => {}
                }
                self.sprite_mut(i).comportamientos[k] = c;
            }
        }
        for &i in &orden {
            let s = self.sprite_mut(i);
            let mut y = s.y;
            for c in &mut s.comportamientos {
                if let Comportamiento::Plataforma(p) = c {
                    p.posttick(&mut y, dt);
                }
            }
            s.y = y;
        }
    }

    /// Bullet.tick.
    fn tick_bala(&mut self, i: usize, b: &mut Bala) {
        if !b.habilitada {
            return;
        }
        let dt = self.dt;
        let angulo = self.sprite(i).angulo;
        if angulo != b.ultimo_angulo {
            if b.fijar_angulo {
                let v = distancia(0.0, 0.0, b.dx, b.dy);
                b.dx = angulo.cos() * v;
                b.dy = angulo.sin() * v;
            }
            b.ultimo_angulo = angulo;
        }
        if b.aceleracion != 0.0 {
            let mut v = distancia(0.0, 0.0, b.dx, b.dy);
            let a = if b.dx == 0.0 && b.dy == 0.0 {
                angulo
            } else {
                angulo_hacia(0.0, 0.0, b.dx, b.dy)
            };
            v += b.aceleracion * dt;
            if v < 0.0 {
                v = 0.0;
            }
            b.dx = a.cos() * v;
            b.dy = a.sin() * v;
        }
        if b.gravedad != 0.0 {
            b.dy += b.gravedad * dt;
        }
        (b.ultima_x, b.ultima_y) = self.posicion(i);
        if b.dx == 0.0 && b.dy == 0.0 {
            return;
        }
        let s = self.sprite_mut(i);
        s.x += b.dx * dt;
        s.y += b.dy * dt;
        b.recorrido += distancia(0.0, 0.0, b.dx * dt, b.dy * dt);
        if b.fijar_angulo {
            s.angulo = angulo_hacia(0.0, 0.0, b.dx, b.dy);
            b.ultimo_angulo = s.angulo;
        }
        if !b.rebota {
            return;
        }
        let Some(solido) = self.solido_que_solapa(i) else {
            return;
        };
        self.registrar_colision(i, solido);
        let v = distancia(0.0, 0.0, b.dx, b.dy);
        let a = self.angulo_rebote(i, b.ultima_x, b.ultima_y);
        b.dx = a.cos() * v;
        b.dy = a.sin() * v;
        let s = self.sprite_mut(i);
        s.x += b.dx * dt;
        s.y += b.dy * dt;
        if b.fijar_angulo {
            s.angulo = a;
            b.ultimo_angulo = a;
        }
        if !self.empujar_fuera(i, b.dx / v, b.dy / v, (2.5 * v * dt).max(30.0)) {
            self.empujar_fuera_cercano(i);
        }
    }

    /// calculateSolidBounceAngle: busca, girando de 5 en 5 grados alrededor de la posición
    /// anterior, la primera dirección libre en cada sentido; la normal está entre las dos, y el
    /// ángulo de rebote es el reflejo de la dirección de llegada.
    fn angulo_rebote(&mut self, i: usize, desde_x: f64, desde_y: f64) -> f64 {
        let (ox, oy) = self.posicion(i);
        let radio = distancia(desde_x, desde_y, ox, oy).max(10.0);
        let inicio = angulo_hacia(desde_x, desde_y, ox, oy);
        let Some(primero) = self.solido_que_solapa(i) else {
            return angulo_acotado(inicio + PI);
        };
        let paso = a_radianes(5.0);
        let mut libre = [angulo_acotado(inicio + PI); 2];
        for (lado, signo) in [(0, -1.0), (1, 1.0)] {
            let mut actual = Some(primero);
            for k in 1..36 {
                let a = inicio + signo * k as f64 * paso;
                self.set_posicion(i, desde_x + a.cos() * radio, desde_y + a.sin() * radio);
                if !self.solapa_con(i, actual) {
                    actual = self.solido_que_solapa(i);
                    if actual.is_none() {
                        libre[lado] = a;
                        break;
                    }
                }
            }
        }
        self.set_posicion(i, ox, oy);
        let [antihorario, horario] = libre;
        if horario == antihorario {
            return horario;
        }
        let mitad = diferencia_angular(horario, antihorario) / 2.0;
        let normal = if en_sentido_horario(horario, antihorario) {
            angulo_acotado(antihorario + mitad + PI)
        } else {
            angulo_acotado(horario + mitad)
        };
        let (vx, vy) = (inicio.cos(), inicio.sin());
        let (nx, ny) = (normal.cos(), normal.sin());
        let p = vx * nx + vy * ny;
        angulo_hacia(0.0, 0.0, vx - 2.0 * p * nx, vy - 2.0 * p * ny)
    }

    /// pushOutSolid: avanza de a un píxel en la dirección (dx, dy) hasta salir de los sólidos y
    /// luego se acerca en fracciones al último que tocó.
    fn empujar_fuera(&mut self, i: usize, dx: f64, dy: f64, distancia: f64) -> bool {
        let distancia = if distancia == 0.0 { 50.0 } else { distancia };
        let (ox, oy) = self.posicion(i);
        let mut ultimo = None;
        let mut tocado = None;
        let mut d = 0.0;
        while d < distancia {
            self.set_posicion(i, ox + dx * d, oy + dy * d);
            if !self.solapa_con(i, ultimo) {
                ultimo = self.solido_que_solapa(i);
                if ultimo.is_some() {
                    tocado = ultimo;
                } else {
                    if let Some(s) = tocado {
                        self.empujar_dentro_fraccional(i, dx, dy, s);
                    }
                    return true;
                }
            }
            d += 1.0;
        }
        self.set_posicion(i, ox, oy);
        false
    }

    /// pushInFractional: búsqueda binaria (1/2 a 1/16 de píxel) hacia el sólido.
    fn empujar_dentro_fraccional(&mut self, i: usize, dx: f64, dy: f64, solido: usize) {
        let mut divisor = 2.0;
        let mut adelante = false;
        let mut solapado = false;
        let (mut mejor_x, mut mejor_y) = self.posicion(i);
        while divisor <= 16.0 {
            let fraccion = 1.0 / divisor;
            divisor *= 2.0;
            let signo = if adelante { 1.0 } else { -1.0 };
            let (x, y) = self.posicion(i);
            self.set_posicion(i, x + dx * fraccion * signo, y + dy * fraccion * signo);
            if self.solapa(i, solido) {
                adelante = true;
                solapado = true;
            } else {
                adelante = false;
                solapado = false;
                (mejor_x, mejor_y) = self.posicion(i);
            }
        }
        if solapado {
            self.set_posicion(i, mejor_x, mejor_y);
        }
    }

    /// pushOutSolidNearest: prueba en espiral (las 8 direcciones, un píxel más por vuelta)
    /// hasta 100 píxeles.
    fn empujar_fuera_cercano(&mut self, i: usize) -> bool {
        let (ox, oy) = self.posicion(i);
        let mut ultimo = self.solido_que_solapa(i);
        if ultimo.is_none() {
            return true;
        }
        let (mut distancia, mut direccion) = (0.0, 0);
        while distancia <= 100.0 {
            let (dx, dy) = match direccion {
                0 => {
                    distancia += 1.0;
                    (0.0, -1.0)
                }
                1 => (1.0, -1.0),
                2 => (1.0, 0.0),
                3 => (1.0, 1.0),
                4 => (0.0, 1.0),
                5 => (-1.0, 1.0),
                6 => (-1.0, 0.0),
                _ => (-1.0, -1.0),
            };
            direccion = (direccion + 1) % 8;
            self.set_posicion(
                i,
                piso_c2(ox + dx * distancia),
                piso_c2(oy + dy * distancia),
            );
            if !self.solapa_con(i, ultimo) {
                ultimo = self.solido_que_solapa(i);
                if ultimo.is_none() {
                    return true;
                }
            }
        }
        self.set_posicion(i, ox, oy);
        false
    }

    /// Turret.tick (sin puntería predictiva ni disparos).
    fn tick_torreta(&mut self, i: usize, t: &mut Torreta) {
        if !t.habilitada {
            return;
        }
        let ahora = self.tiempo();
        let (x, y) = self.posicion(i);
        let alcance2 = t.alcance * t.alcance;
        let en_alcance = |m: &Mundo, j: usize| {
            let (tx, ty) = m.posicion(j);
            let (dx, dy) = (tx - x, ty - y);
            dx * dx + dy * dy <= alcance2
        };
        let mut objetivo = t.objetivo.and_then(|u| self.por_uid(u));
        if objetivo.is_some_and(|j| !en_alcance(self, j)) {
            objetivo = None;
        }
        if ahora >= t.ultimo_chequeo + 0.1 {
            t.ultimo_chequeo = ahora;
            let tipo = self.instancias[i].tipo();
            let candidatas: Vec<usize> = self
                .objetivos
                .get(&tipo)
                .map(|tipos| tipos.iter().flat_map(|&tt| self.de(tt)).collect())
                .unwrap_or_default();
            if t.modo == 0 && objetivo.is_none() {
                objetivo = candidatas.into_iter().find(|&j| en_alcance(self, j));
            } else if t.modo == 1 {
                objetivo = None;
                let mut mejor = alcance2;
                for j in candidatas {
                    let (tx, ty) = self.posicion(j);
                    let d = (x - tx) * (x - tx) + (y - ty) * (y - ty);
                    if d < mejor {
                        objetivo = Some(j);
                        mejor = d;
                    }
                }
            }
        }
        t.objetivo = objetivo.map(|j| self.uid(j));
        if let Some(j) = objetivo {
            let (tx, ty) = self.posicion(j);
            let a = angulo_hacia(x, y, tx, ty);
            if t.rotar {
                let paso = t.velocidad_giro * self.dt;
                let s = self.sprite_mut(i);
                s.angulo = rotar_angulo(s.angulo, a, paso);
            }
        }
    }

    /// Wrap.tick: si la caja salió del todo por un borde, aparece por el contrario.
    fn tick_envolver(&mut self, i: usize, e: &Envolver) {
        let (caja, _) = {
            let s = self.sprite(i);
            s.cajas(&self.proyecto.tipos[s.tipo])
        };
        // Modo 0: los bordes del layout; modo 1: los de la vista de la capa.
        let (izq, der, arr, aba) = if e.modo == 0 {
            let l = &self.proyecto.layouts[self.layout];
            (0.0, l.ancho, 0.0, l.alto)
        } else {
            let (izq, arr) = self.vista_capa(self.sprite(i).capa);
            (
                izq,
                izq + self.proyecto.ancho,
                arr,
                arr + self.proyecto.alto,
            )
        };
        let s = self.sprite_mut(i);
        if caja.right < izq {
            s.x = der - 1.0 + (s.x - caja.left);
        } else if caja.left > der {
            s.x = izq + 1.0 - (caja.right - s.x);
        } else if caja.bottom < arr {
            s.y = aba - 1.0 + (s.y - caja.top);
        } else if caja.top > aba {
            s.y = arr + 1.0 - (caja.bottom - s.y);
        }
    }

    // --- dibujo ------------------------------------------------------------------------------

    /// Layout.draw: cada capa visible (las opacas se pintan de su color) con sus instancias en
    /// orden.
    pub fn dibujar(&self) {
        let layout = &self.proyecto.layouts[self.layout];
        for (c, capa) in layout.capas.iter().enumerate() {
            if !self.capas_visibles.get(c).copied().unwrap_or(capa.visible) {
                continue;
            }
            if !capa.transparente {
                clear_background(capa.fondo);
            }
            let (izq, arr) = self.vista_capa(c);
            let desplazamiento = DVec2::new(-izq, -arr);
            for inst in &self.instancias {
                if inst.capa() != c || inst.destruido() {
                    continue;
                }
                match inst {
                    Inst::Sprite(s) => {
                        s.dibujar(&self.proyecto.tipos[s.tipo], &self.texturas, desplazamiento)
                    }
                    Inst::Texto(t) => t.dibujar(&self.fuentes, desplazamiento),
                    Inst::Particulas(p) => p.dibujar(&self.texturas, desplazamiento),
                }
            }
        }
    }
}
