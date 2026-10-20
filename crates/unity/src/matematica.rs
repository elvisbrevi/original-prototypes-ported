// Las funciones de Mathf y Vector3 que usan los scripts, traducidas del C# de UnityEngine.dll
// (Unity 5.1.2, decompilado con ILSpy): en Unity 5 son código administrado, no nativo.

use macroquad::prelude::Vec3;

/// Mathf.Sign: 1 para cero y los positivos.
pub fn signo(f: f32) -> f32 {
    if f >= 0.0 { 1.0 } else { -1.0 }
}

/// Vector3.ClampMagnitude.
pub fn limitar_largo(v: Vec3, largo: f32) -> Vec3 {
    if v.length_squared() > largo * largo {
        v.normalize() * largo
    } else {
        v
    }
}

/// Vector3.MoveTowards.
pub fn mover_hacia(actual: Vec3, destino: Vec3, paso: f32) -> Vec3 {
    let v = destino - actual;
    let largo = v.length();
    if largo <= paso || largo == 0.0 {
        destino
    } else {
        actual + v / largo * paso
    }
}

/// Vector3.SmoothDamp(current, target, ref currentVelocity, smoothTime, maxSpeed, deltaTime).
pub fn suavizar(
    actual: Vec3,
    destino: Vec3,
    velocidad: &mut Vec3,
    tiempo: f32,
    rapidez_maxima: f32,
    dt: f32,
) -> Vec3 {
    let tiempo = if 0.0001 > tiempo { 0.0001 } else { tiempo };
    let num = 2.0 / tiempo;
    let num2 = num * dt;
    let num3 = 1.0 / (1.0 + num2 + 0.48 * num2 * num2 + 0.235 * num2 * num2 * num2);
    let original = destino;
    let vector = limitar_largo(actual - destino, rapidez_maxima * tiempo);
    let destino = actual - vector;
    let vector3 = (*velocidad + num * vector) * dt;
    *velocidad = (*velocidad - num * vector3) * num3;
    let mut salida = destino + (vector + vector3) * num3;
    if (original - actual).dot(salida - original) > 0.0 {
        salida = original;
        *velocidad = (salida - original) / dt;
    }
    salida
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec3;

    #[test]
    fn suavizar_se_acerca_sin_pasarse_y_con_dt_cero_no_se_mueve() {
        let mut v = Vec3::ZERO;
        let mut p = Vec3::ZERO;
        let destino = vec3(10.0, 0.0, 0.0);
        for _ in 0..200 {
            p = suavizar(p, destino, &mut v, 0.5, f32::INFINITY, 1.0 / 60.0);
            assert!(p.x <= 10.0);
        }
        assert!((p.x - 10.0).abs() < 1e-3);
        let quieto = suavizar(
            vec3(1.0, 2.0, 3.0),
            destino,
            &mut v,
            0.5,
            f32::INFINITY,
            0.0,
        );
        assert_eq!(quieto, vec3(1.0, 2.0, 3.0));
    }

    #[test]
    fn mover_hacia_y_signo() {
        assert_eq!(
            mover_hacia(Vec3::ZERO, vec3(3.0, 4.0, 0.0), 1.0),
            vec3(0.6, 0.8, 0.0)
        );
        assert_eq!(
            mover_hacia(Vec3::ZERO, vec3(3.0, 4.0, 0.0), 9.0),
            vec3(3.0, 4.0, 0.0)
        );
        assert_eq!(signo(0.0), 1.0);
        assert_eq!(signo(-0.5), -1.0);
    }
}
