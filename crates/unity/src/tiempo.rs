// Time de Unity 4: deltaTime es el tiempo real del frame (como mucho Maximum Allowed Timestep)
// multiplicado por timeScale, y los FixedUpdate van a paso fijo detrás de Time.time. Con
// timeScale = 0 no corre ningún FixedUpdate.

use crate::datos::AjustesTiempo;

pub struct Tiempo {
    /// Time.timeScale.
    pub escala: f32,
    /// Time.deltaTime del frame (escalado).
    pub delta: f32,
    /// Time.time.
    pub tiempo: f64,
    /// Time.fixedTime.
    pub tiempo_fijo: f64,
    /// Time.fixedDeltaTime.
    pub paso_fijo: f32,
    /// Time.maximumDeltaTime.
    pub paso_maximo: f32,
    /// Time.frameCount.
    pub frame: u64,
}

impl Tiempo {
    pub fn nuevo(ajustes: &AjustesTiempo) -> Self {
        Self {
            escala: ajustes.escala,
            delta: 0.0,
            tiempo: 0.0,
            tiempo_fijo: 0.0,
            paso_fijo: ajustes.paso_fijo,
            paso_maximo: ajustes.paso_maximo,
            frame: 0,
        }
    }

    /// Empieza un frame que duró `real` segundos.
    pub fn nuevo_frame(&mut self, real: f32) {
        self.frame += 1;
        self.delta = real.min(self.paso_maximo) * self.escala;
        self.tiempo += self.delta as f64;
    }

    /// Si toca otro FixedUpdate en este frame (y avanza Time.fixedTime).
    pub fn siguiente_paso_fijo(&mut self) -> bool {
        let paso = self.paso_fijo as f64;
        if self.tiempo_fijo + paso <= self.tiempo {
            self.tiempo_fijo += paso;
            true
        } else {
            false
        }
    }
}
