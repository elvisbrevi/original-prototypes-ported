// El InputManager de Unity: Input.GetAxis con los ejes de teclas (el valor va hacia ±1 a
// «sensitivity» unidades por segundo y vuelve a 0 a «gravity»; con «snap», al apretar la tecla
// contraria salta primero a 0) e Input.GetButton, GetButtonDown y GetButtonUp. Un eje puede tener
// varias entradas con el mismo nombre (teclado y mando): GetAxis devuelve la de mayor valor
// absoluto, y un botón está apretado si lo está alguna de sus entradas.

use std::collections::HashSet;

use crate::datos3d::Eje;
use crate::matematica::signo;

pub struct Entrada {
    ejes: Vec<(Eje, f32)>,
    abajo: HashSet<String>,
    pulsadas: HashSet<String>,
    /// Las teclas apretadas en el frame anterior.
    antes: HashSet<String>,
}

/// Las teclas que el juego tiene apretadas y las que se apretaron en este frame, con los nombres
/// de Unity («left», «up», «a», «w»...).
pub struct Teclas<'a> {
    pub abajo: &'a [&'a str],
    pub pulsadas: &'a [&'a str],
}

fn mover_hacia(actual: f32, objetivo: f32, paso: f32) -> f32 {
    if (objetivo - actual).abs() <= paso {
        objetivo
    } else {
        actual + signo(objetivo - actual) * paso
    }
}

impl Entrada {
    pub fn nueva(ejes: &[Eje]) -> Self {
        Self {
            ejes: ejes.iter().map(|e| (e.clone(), 0.0)).collect(),
            abajo: HashSet::new(),
            pulsadas: HashSet::new(),
            antes: HashSet::new(),
        }
    }

    /// Al empezar el frame, con el tiempo real que pasó.
    pub fn actualizar(&mut self, teclas: &Teclas, dt: f32) {
        self.antes = std::mem::take(&mut self.abajo);
        self.abajo = teclas.abajo.iter().map(|t| t.to_string()).collect();
        self.pulsadas = teclas.pulsadas.iter().map(|t| t.to_string()).collect();
        for (e, valor) in &mut self.ejes {
            if e.tipo != 0 {
                continue;
            }
            let tecla = |n: &str| !n.is_empty() && self.abajo.contains(n);
            let pos = tecla(&e.positivo) || tecla(&e.positivo_alt);
            let neg = tecla(&e.negativo) || tecla(&e.negativo_alt);
            let objetivo = match (pos, neg) {
                (true, false) => 1.0,
                (false, true) => -1.0,
                _ => 0.0,
            };
            if objetivo != 0.0 {
                if e.salto && *valor != 0.0 && signo(*valor) != signo(objetivo) {
                    *valor = 0.0;
                }
                *valor = mover_hacia(*valor, objetivo, e.sensibilidad * dt);
            } else {
                *valor = mover_hacia(*valor, 0.0, e.gravedad * dt);
            }
        }
    }

    /// Input.GetAxis.
    pub fn eje(&self, nombre: &str) -> f32 {
        let mut mejor = 0.0f32;
        for (e, v) in &self.ejes {
            if e.nombre != nombre {
                continue;
            }
            let v = if v.abs() < e.zona_muerta { 0.0 } else { *v };
            let v = if e.invertir { -v } else { v };
            if v.abs() > mejor.abs() {
                mejor = v;
            }
        }
        mejor
    }

    /// Las teclas positivas de las entradas del botón.
    fn teclas_boton<'a>(&'a self, nombre: &'a str) -> impl Iterator<Item = [&'a str; 2]> + 'a {
        self.ejes
            .iter()
            .filter(move |(e, _)| e.nombre == nombre)
            .map(|(e, _)| [e.positivo.as_str(), e.positivo_alt.as_str()])
    }

    /// Input.GetButton: alguna tecla positiva del botón está apretada.
    pub fn boton(&self, nombre: &str) -> bool {
        self.teclas_boton(nombre)
            .flatten()
            .any(|t| !t.is_empty() && self.abajo.contains(t))
    }

    /// Input.GetButtonUp: alguna entrada del botón estaba apretada en el frame anterior y ya no.
    pub fn boton_soltado(&self, nombre: &str) -> bool {
        self.teclas_boton(nombre).any(|teclas| {
            let en = |s: &HashSet<String>| teclas.iter().any(|t| !t.is_empty() && s.contains(*t));
            en(&self.antes) && !en(&self.abajo)
        })
    }

    /// Input.GetButtonDown: alguna tecla positiva del botón se apretó en este frame.
    pub fn boton_pulsado(&self, nombre: &str) -> bool {
        self.ejes.iter().any(|(e, _)| {
            e.nombre == nombre
                && [&e.positivo, &e.positivo_alt]
                    .iter()
                    .any(|t| !t.is_empty() && self.pulsadas.contains(t.as_str()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn horizontal() -> Eje {
        Eje {
            nombre: "Horizontal".into(),
            negativo: "left".into(),
            positivo: "right".into(),
            negativo_alt: "a".into(),
            positivo_alt: "d".into(),
            gravedad: 3.0,
            zona_muerta: 0.001,
            sensibilidad: 3.0,
            salto: true,
            invertir: false,
            tipo: 0,
        }
    }

    #[test]
    fn el_eje_sube_a_su_ritmo_y_salta_a_cero_al_cambiar() {
        let mut e = Entrada::nueva(&[horizontal()]);
        let derecha = Teclas {
            abajo: &["right"],
            pulsadas: &[],
        };
        e.actualizar(&derecha, 0.1);
        assert!((e.eje("Horizontal") - 0.3).abs() < 1e-6);
        for _ in 0..10 {
            e.actualizar(&derecha, 0.1);
        }
        assert_eq!(e.eje("Horizontal"), 1.0);
        e.actualizar(
            &Teclas {
                abajo: &["a"],
                pulsadas: &["a"],
            },
            0.1,
        );
        assert!((e.eje("Horizontal") + 0.3).abs() < 1e-6);
    }

    #[test]
    fn los_botones_se_sueltan_un_frame_despues() {
        let fire2 = Eje {
            nombre: "Fire2".into(),
            negativo: String::new(),
            positivo: "left alt".into(),
            negativo_alt: String::new(),
            positivo_alt: "mouse 1".into(),
            gravedad: 1000.0,
            zona_muerta: 0.001,
            sensibilidad: 1000.0,
            salto: false,
            invertir: false,
            tipo: 0,
        };
        let mut e = Entrada::nueva(&[fire2]);
        let apretada = Teclas {
            abajo: &["mouse 1"],
            pulsadas: &["mouse 1"],
        };
        e.actualizar(&apretada, 0.02);
        assert!(e.boton("Fire2") && e.boton_pulsado("Fire2") && !e.boton_soltado("Fire2"));
        let nada = Teclas {
            abajo: &[],
            pulsadas: &[],
        };
        e.actualizar(&nada, 0.02);
        assert!(!e.boton("Fire2") && e.boton_soltado("Fire2"));
        e.actualizar(&nada, 0.02);
        assert!(!e.boton_soltado("Fire2"));
    }
}
