// Piezas de las hojas de eventos que guardan estado entre ticks: System.Wait, la memoria de
// colisiones de "On collision with another object" y el cursor del plugin Mouse.

use std::collections::HashMap;

use macroquad::miniquad::CursorIcon;
use macroquad::miniquad::window::{set_mouse_cursor, show_mouse};

use crate::mundo::Mundo;

/// Una acción Wait pendiente: cuándo se reanuda, qué acciones siguen (lo decide cada juego) y
/// las instancias que tenía elegidas el evento (sus uid).
pub struct Espera<K> {
    pub tiempo: f64,
    pub seguir: K,
    pub elegidas: Vec<u32>,
}

/// System.Wait: las esperas viven en el objeto System, así que sobreviven a los cambios de
/// layout (salvo al reiniciar el mismo layout, que las borra).
pub struct Esperas<K> {
    lista: Vec<Espera<K>>,
}

impl<K> Default for Esperas<K> {
    fn default() -> Self {
        Self { lista: Vec::new() }
    }
}

impl<K> Esperas<K> {
    /// Wait(segundos): con segundos negativos no espera (devuelve false y las acciones siguen).
    pub fn esperar(&mut self, tiempo: f64, segundos: f64, seguir: K, elegidas: Vec<u32>) -> bool {
        if segundos < 0.0 {
            return false;
        }
        self.lista.push(Espera {
            tiempo: tiempo + segundos,
            seguir,
            elegidas,
        });
        true
    }

    /// runWaits: las esperas cumplidas, en orden, al principio del tick.
    pub fn cumplidas(&mut self, tiempo: f64) -> Vec<Espera<K>> {
        let (cumplidas, pendientes) = std::mem::take(&mut self.lista)
            .into_iter()
            .partition(|e| e.tiempo <= tiempo);
        self.lista = pendientes;
        cumplidas
    }

    pub fn vaciar(&mut self) {
        self.lista.clear();
    }
}

/// La memoria de la condición "On collision with another object" (collmemory): cada condición
/// recuerda en qué tick vio solapadas a cada pareja de instancias.
#[derive(Default)]
pub struct MemoriaColision {
    parejas: HashMap<(u32, u32), u64>,
}

impl MemoriaColision {
    /// Las parejas (izquierda, derecha) que empiezan a chocar en este tick: solapadas (o con una
    /// colisión registrada por un comportamiento) y que no lo estaban en el tick anterior. El
    /// original vuelve a ejecutar el resto del evento por cada pareja ("retrigger"), en orden.
    pub fn nuevas(
        &mut self,
        mundo: &Mundo,
        izquierda: &[usize],
        tipo_derecha: usize,
    ) -> Vec<(usize, usize)> {
        let tick = mundo.ticks;
        let mut nuevas = Vec::new();
        for &a in izquierda {
            for b in mundo.de(tipo_derecha) {
                let (ua, ub) = (mundo.uid(a), mundo.uid(b));
                let clave = if ua < ub { (ua, ub) } else { (ub, ua) };
                if mundo.solapa(a, b) || mundo.colision_registrada(a, b) {
                    let correr = match self.parejas.get(&clave) {
                        None => true,
                        Some(&t) => t + 1 < tick,
                    };
                    self.parejas.insert(clave, tick);
                    if correr {
                        nuevas.push((a, b));
                    }
                } else {
                    self.parejas.remove(&clave);
                }
            }
        }
        nuevas
    }
}

/// Mouse.SetCursor: "auto pointer text crosshair move help wait none" (solo cambia si es otro).
#[derive(Default)]
pub struct Cursor {
    actual: Option<i32>,
}

impl Cursor {
    pub fn poner(&mut self, opcion: i32) {
        if self.actual == Some(opcion) {
            return;
        }
        self.actual = Some(opcion);
        show_mouse(opcion != 7);
        set_mouse_cursor(match opcion {
            1 => CursorIcon::Pointer,
            2 => CursorIcon::Text,
            3 => CursorIcon::Crosshair,
            4 => CursorIcon::Move,
            5 => CursorIcon::Help,
            6 => CursorIcon::Wait,
            _ => CursorIcon::Default,
        });
    }
}
