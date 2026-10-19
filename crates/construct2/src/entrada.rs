// Los toques y el ratón entre ticks, como los eventos del DOM que recibe el plugin Touch (que
// también toma el ratón si su propiedad "Use mouse input" está activa) y la posición que guarda
// el plugin Mouse.

use std::collections::HashSet;

use macroquad::math::DVec2;
use macroquad::prelude::*;

#[derive(Default)]
pub struct Toques {
    vistos: HashSet<u64>,
}

#[derive(Default)]
pub struct Entrada {
    /// Toques que empezaron desde el último fotograma, en coordenadas del layout.
    pub inicios: Vec<DVec2>,
    /// Toques activos.
    pub activos: Vec<DVec2>,
    /// Última posición del ratón.
    pub raton: DVec2,
    /// Clic izquierdo en este fotograma (Mouse "On object clicked").
    pub clic: Option<DVec2>,
}

impl Toques {
    /// `a_layout` pasa de coordenadas de la ventana a las del layout.
    pub fn leer(&mut self, a_layout: impl Fn(Vec2) -> Vec2, usar_raton: bool) -> Entrada {
        let mut e = Entrada::default();
        for t in touches() {
            let p = a_layout(t.position).as_dvec2();
            if self.vistos.insert(t.id) {
                e.inicios.push(p);
            }
            if matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                self.vistos.remove(&t.id);
            } else {
                e.activos.push(p);
            }
        }
        e.raton = a_layout(mouse_position().into()).as_dvec2();
        if is_mouse_button_pressed(MouseButton::Left) {
            e.clic = Some(e.raton);
        }
        if usar_raton {
            if is_mouse_button_pressed(MouseButton::Left) {
                e.inicios.push(e.raton);
            }
            if is_mouse_button_down(MouseButton::Left) {
                e.activos.push(e.raton);
            }
        }
        e
    }
}
