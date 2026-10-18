// Port de Engine.Fisicas.Fisica: gravedad y colisiones por rectángulos. Solo los objetos con
// Gravedad chocan; usan el cuerpo calculado en el paso anterior.

use xna::clamp;

use crate::motor::objeto::{Colision, Objeto};

/// Fisica.aplicarFisicas sobre objetos[i].
pub fn aplicar_fisicas(objetos: &mut [Objeto], i: usize) {
    aplicar_gravedad(&mut objetos[i]);
    aplicar_colision(objetos, i);
}

fn aplicar_gravedad(o: &mut Objeto) {
    if !o.gravedad {
        return;
    }
    if o.colision_superior {
        o.fuerza_gravedad = 0.0;
        return;
    }
    o.posicion.y += o.fuerza_gravedad;
    if o.fuerza_gravedad < 8.0 {
        o.fuerza_gravedad += 0.5;
    }
}

fn aplicar_colision(objetos: &mut [Objeto], i: usize) {
    for j in 0..objetos.len() {
        let (a, b) = (objetos[i].cuerpo, objetos[j].cuerpo);
        if !a.intersects(&b) || j == i || !objetos[i].gravedad || !objetos[j].activo {
            continue;
        }
        let otro = &objetos[j];
        let (colision, movimento, ida_x, ida_y, inicio, destino, velocidad_otro) = (
            otro.colision,
            otro.movimento,
            otro.ida_x,
            otro.ida_y,
            otro.inicio,
            otro.destino,
            otro.velocidad,
        );
        let (mut izquierda_otro, mut derecha_otro, mut superior_otro, mut inferior_otro) =
            (false, false, false, false);
        let o = &mut objetos[i];
        let v = o.velocidad;
        if matches!(colision, Colision::Lateral | Colision::Total) {
            if a.left() <= b.right()
                && a.left() as f32 + (v + 1.0) > b.right() as f32
                && a.bottom() > b.top()
                && a.top() < b.bottom()
            {
                o.posicion.x = clamp(o.posicion.x, b.right() as f32 - v, a.right() as f32 + v);
                o.colision_izquierda = true;
                derecha_otro = true;
            }
            if a.right() >= b.left()
                && a.right() as f32 - (v + 1.0) < b.left() as f32
                && a.bottom() > b.top()
                && a.top() < b.bottom()
            {
                o.posicion.x = clamp(
                    o.posicion.x,
                    a.left() as f32 - v,
                    b.left() as f32 - (a.width as f32 - v),
                );
                o.colision_derecha = true;
                izquierda_otro = true;
            }
        }
        if matches!(colision, Colision::Superior | Colision::Total)
            && a.bottom() >= b.top()
            && a.top() <= b.top() - (a.height - 9)
            && a.right() - 5 > b.left()
            && a.left() + 5 < b.right()
        {
            o.posicion.y = clamp(o.posicion.y, -100.0, (b.top() - (a.height - 1)) as f32);
            o.colision_superior = true;
            inferior_otro = true;
            // Una plataforma que se mueve arrastra al objeto.
            if movimento {
                if ida_y && destino.y - inicio.y != 0.0 {
                    o.posicion.y += velocidad_otro;
                } else if destino.y - inicio.y != 0.0 {
                    o.posicion.y -= velocidad_otro;
                }
                if ida_x && destino.x - inicio.x != 0.0 {
                    o.posicion.x += velocidad_otro;
                } else if destino.x - inicio.x != 0.0 {
                    o.posicion.x -= velocidad_otro;
                }
            }
        }
        if matches!(colision, Colision::Inferior | Colision::Total)
            && a.top() <= b.bottom()
            && a.bottom() >= b.bottom() + (a.height - 9)
            && a.right() - 5 > b.left()
            && a.left() + 5 < b.right()
        {
            o.posicion.y = clamp(o.posicion.y, (b.bottom() - 1) as f32, a.bottom() as f32);
            o.colision_inferior = true;
            superior_otro = true;
        }
        let otro = &mut objetos[j];
        otro.colision_izquierda |= izquierda_otro;
        otro.colision_derecha |= derecha_otro;
        otro.colision_superior |= superior_otro;
        otro.colision_inferior |= inferior_otro;
    }
}
