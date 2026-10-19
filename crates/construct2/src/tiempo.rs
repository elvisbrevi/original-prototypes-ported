// El tiempo del runtime: el dt de cada tick, la suma de Kahan con la que acumula el tiempo de
// juego y los relojes de animación, y la condición "Every X seconds".

/// cr.KahanAdder: suma compensada.
#[derive(Clone, Copy, Debug, Default)]
pub struct Kahan {
    pub suma: f64,
    c: f64,
}

impl Kahan {
    pub fn sumar(&mut self, v: f64) {
        let y = v - self.c;
        let t = self.suma + y;
        self.c = (t - self.suma) - y;
        self.suma = t;
    }
}

/// El dt del runtime a partir del tiempo real del fotograma: 0 en el primer tick o si pasó más
/// de medio segundo (pestaña en segundo plano), y como mucho 0,1 s.
pub fn dt_del_runtime(dt_real: f64, primero: bool) -> f64 {
    if primero || dt_real > 0.5 {
        0.0
    } else if dt_real > 0.1 {
        0.1
    } else {
        dt_real
    }
}

/// System "Every X seconds": cada condición guarda su último disparo (Every_lastTime) y los
/// segundos con los que se evaluó la primera vez; si se retrasa más de 40 ms, se recoloca.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cada {
    ultimo: f64,
    segundos: Option<f64>,
}

impl Cada {
    pub fn cumple(&mut self, segundos: f64, tiempo: f64) -> bool {
        let ultimo = self.ultimo;
        let s = *self.segundos.get_or_insert(segundos);
        if tiempo >= ultimo + s {
            self.ultimo = ultimo + s;
            if tiempo >= self.ultimo + 0.04 {
                self.ultimo = tiempo;
            }
            self.segundos = Some(segundos);
            return true;
        }
        if tiempo < ultimo - 0.1 {
            self.ultimo = tiempo;
        }
        false
    }
}
