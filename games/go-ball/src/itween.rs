// Lo que usa Go Ball de iTween (la versión que viene en su Assembly-CSharp.dll): RotateBy con
// «x», «easeType», «loopType» y «delay» (Objetivo.Start). iTween pasa las claves a minúsculas
// (CleanArgs), así que «easeType» y «loopType» valen; el tiempo es el por defecto (1 s) y el
// espacio, Space.Self.

use macroquad::prelude::Vec3;

/// easeInOutBack de iTween.
pub fn ease_in_out_back(inicio: f32, fin: f32, valor: f32) -> f32 {
    let mut s = 1.70158f32;
    let fin = fin - inicio;
    let mut valor = valor / 0.5;
    if valor < 1.0 {
        s *= 1.525;
        return fin / 2.0 * (valor * valor * ((s + 1.0) * valor - s)) + inicio;
    }
    valor -= 2.0;
    s *= 1.525;
    fin / 2.0 * (valor * valor * ((s + 1.0) * valor + s) + 2.0) + inicio
}

/// Un iTween RotateBy con loopType pingPong.
pub struct RotarPor {
    /// vector3s[0], [1] y [3]: los ángulos de partida, de llegada y los últimos aplicados.
    inicio: Vec3,
    fin: Vec3,
    ultimo: Vec3,
    tiempo: f32,
    retraso: f32,
    corriendo: bool,
    /// Lo que falta de la espera (Start o TweenRestart), en tiempo escalado.
    espera: Option<f32>,
    porcentaje: f32,
    transcurrido: f32,
    reverso: bool,
}

impl RotarPor {
    /// iTween.RotateBy(objeto, Hash("x", x, "delay", retraso, ...)): se lanza y empieza después
    /// del retraso.
    pub fn nuevo(x: f32, retraso: f32) -> Self {
        Self {
            inicio: Vec3::ZERO,
            fin: Vec3::new(360.0 * x, 0.0, 0.0),
            ultimo: Vec3::ZERO,
            tiempo: 1.0,
            retraso,
            corriendo: false,
            espera: Some(retraso),
            porcentaje: 0.0,
            transcurrido: 0.0,
            reverso: false,
        }
    }

    /// El paso de las corrutinas (después de Update): cuando termina la espera, TweenStart.
    /// `angulos` es transform.eulerAngles, que GenerateRotateByTargets toma la primera vez.
    pub fn corrutina(&mut self, dt: f32, angulos: Vec3, primera: &mut bool) {
        if let Some(e) = self.espera.as_mut() {
            *e -= dt;
            if *e <= 0.0 {
                self.espera = None;
                if *primera {
                    // GenerateRotateByTargets.
                    let x = self.fin.x;
                    self.inicio = angulos;
                    self.ultimo = angulos;
                    self.fin = angulos + Vec3::new(x, 0.0, 0.0);
                    *primera = false;
                }
                self.corriendo = true;
            }
        }
    }

    /// ApplyRotateAddTargets: devuelve lo que hay que girar (transform.Rotate en Space.Self).
    fn aplicar(&mut self) -> Vec3 {
        let p = self.porcentaje;
        let actual = Vec3::new(
            ease_in_out_back(self.inicio.x, self.fin.x, p),
            ease_in_out_back(self.inicio.y, self.fin.y, p),
            ease_in_out_back(self.inicio.z, self.fin.z, p),
        );
        let giro = actual - self.ultimo;
        self.ultimo = actual;
        giro
    }

    /// iTween.Update: devuelve lo que hay que girar en este frame.
    pub fn update(&mut self, dt: f32) -> Vec3 {
        if !self.corriendo {
            return Vec3::ZERO;
        }
        let sigue = if self.reverso {
            self.porcentaje > 0.0
        } else {
            self.porcentaje < 1.0
        };
        if sigue {
            // TweenUpdate: apply y UpdatePercentage.
            let giro = self.aplicar();
            self.transcurrido += dt;
            self.porcentaje = if self.reverso {
                1.0 - self.transcurrido / self.tiempo
            } else {
                self.transcurrido / self.tiempo
            };
            giro
        } else {
            // TweenComplete y TweenLoop con pingPong: vuelve a esperar el retraso.
            self.corriendo = false;
            self.porcentaje = if self.porcentaje > 0.5 { 1.0 } else { 0.0 };
            let giro = self.aplicar();
            self.reverso = !self.reverso;
            self.transcurrido = 0.0;
            self.espera = Some(self.retraso);
            giro
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_curva_va_de_inicio_a_fin_pasandose_un_poco() {
        assert_eq!(ease_in_out_back(0.0, 90.0, 0.0), 0.0);
        assert!((ease_in_out_back(0.0, 90.0, 1.0) - 90.0).abs() < 1e-4);
        assert!(ease_in_out_back(0.0, 90.0, 0.1) < 0.0);
        assert!(ease_in_out_back(0.0, 90.0, 0.9) > 90.0);
    }

    #[test]
    fn va_y_vuelve_con_espera() {
        let mut t = RotarPor::nuevo(0.25, 0.4);
        let mut primera = true;
        let mut total = 0.0;
        for _ in 0..30 {
            t.corrutina(0.02, Vec3::ZERO, &mut primera);
            total += t.update(0.02).x;
        }
        // 0,6 s: la espera (0,4) y parte de la ida.
        assert!(total.abs() < 90.0);
        for _ in 0..60 {
            total += t.update(0.02).x;
            t.corrutina(0.02, Vec3::ZERO, &mut primera);
        }
        assert!((total - 90.0).abs() < 1e-3, "{total}");
    }
}
