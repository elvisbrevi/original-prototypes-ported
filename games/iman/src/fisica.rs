// Port literal de Iman.Fisicas.Fisica. El orden de las comprobaciones, las conversiones (int)
// y el Clamp de XNA se mantienen porque definen cómo se siente el juego original.

use crate::objetos::{Colision, Movimiento, Objeto, TipoObjeto};
use xna::{clamp, int};

pub fn actualizar(objetos: &mut [Objeto]) {
    aplicar_colision(objetos);
    aplicar_gravedad(objetos);
    aplicar_magnetismo(objetos);
}

fn aplicar_colision(cuerpos: &mut [Objeto]) {
    for c in cuerpos.iter_mut() {
        c.colision_superior = false;
        c.colision_inferior = false;
        c.colision_derecha = false;
        c.colision_izquierda = false;
    }
    for i in 0..cuerpos.len() {
        // cuerpo2 nunca se modifica dentro del bucle interior (cuerpo3 != cuerpo2).
        let c2 = &cuerpos[i];
        let (tipo2, colision2, rect2) = (c2.tipo_objeto, c2.colision, c2.cuerpo);
        let (ida2, movimiento2, velocidad2) = (c2.ida, c2.movimiento, c2.velocidad as f32);
        for (j, c3) in cuerpos.iter_mut().enumerate() {
            if !c3.cuerpo.intersects(&rect2) || tipo2 != TipoObjeto::Plataforma || i == j {
                continue;
            }
            let margen = c3.fuerza_gravedad + c3.velocidad as f32;
            if colision2 == Colision::Lateral || colision2 == Colision::Total {
                if c3.posicion.x >= rect2.x as f32 + (rect2.width as f32 - margen) - 10.0
                    && c3.posicion.y + c3.cuerpo.height as f32 > rect2.y as f32 + margen
                {
                    c3.colision_izquierda = true;
                    c3.posicion.x = int(clamp(
                        c3.posicion.x,
                        rect2.right() as f32,
                        c3.cuerpo.right() as f32,
                    )) as f32;
                }
                if c3.posicion.x + c3.cuerpo.width as f32 <= rect2.x as f32 + margen + 10.0
                    && c3.posicion.y + c3.cuerpo.height as f32 > rect2.y as f32 + margen
                {
                    c3.colision_derecha = true;
                    c3.posicion.x = int(clamp(
                        c3.posicion.x,
                        0.0,
                        (rect2.left() - c3.cuerpo.width) as f32,
                    )) as f32;
                }
            }
            if (colision2 == Colision::Superior || colision2 == Colision::Total)
                && c3.posicion.y + c3.cuerpo.height as f32
                    <= rect2.y as f32 + (c3.fuerza_gravedad + 1.0)
                && c3.posicion.x + c3.cuerpo.width as f32 > rect2.x as f32
            {
                c3.posicion.y = int(clamp(
                    c3.posicion.y,
                    0.0,
                    (rect2.y - c3.cuerpo.height - c3.velocidad) as f32,
                )) as f32;
                c3.colision_superior = true;
                // Arrastre de las plataformas móviles.
                let sentido = if ida2 { 1.0 } else { -1.0 };
                match movimiento2 {
                    Movimiento::Vertical => c3.posicion.y += sentido * velocidad2,
                    Movimiento::Horizontal => c3.posicion.x += sentido * velocidad2,
                    Movimiento::Ninguno => {}
                }
            }
            if (colision2 == Colision::Inferior || colision2 == Colision::Total)
                && c3.posicion.y >= rect2.y as f32 + (rect2.height as f32 - margen)
                && c3.posicion.x + c3.cuerpo.width as f32 > rect2.x as f32
            {
                c3.posicion.y = int(clamp(c3.posicion.y, (rect2.bottom() + 10) as f32, 0.0)) as f32;
                c3.colision_inferior = true;
            }
        }
    }
}

fn aplicar_gravedad(objetos: &mut [Objeto]) {
    use crate::objetos::Estado::Saltando;
    for o in objetos.iter_mut().filter(|o| o.gravedad) {
        if o.estado == Saltando {
            o.fuerza_gravedad = 0.0;
        } else {
            o.posicion.y += o.fuerza_gravedad;
            if o.fuerza_gravedad <= 8.0 {
                o.fuerza_gravedad += 1.0;
            }
        }
    }
}

/// El imán atrae cajas (plataformas con gravedad) en horizontal, y eleva al niño
/// hacia plataformas móviles que estén por encima de él.
fn aplicar_magnetismo(objetos: &mut [Objeto]) {
    let Some(ultimo) = objetos.len().checked_sub(1) else {
        return;
    };
    for k in 0..objetos.len() {
        let nino = &objetos[ultimo];
        let (iman, usando_iman, cuerpo_nino, colision_inferior_nino) = (
            nino.iman,
            nino.usando_iman,
            nino.cuerpo,
            nino.colision_inferior,
        );
        let o = &mut objetos[k];
        if o.tipo_objeto != TipoObjeto::Plataforma || !o.cuerpo.intersects(&iman) || !usando_iman {
            continue;
        }
        if o.gravedad {
            if o.cuerpo.right() < cuerpo_nino.left() && !o.colision_derecha {
                o.posicion.x += 5.0;
            } else if o.cuerpo.left() > cuerpo_nino.right() && !o.colision_izquierda {
                o.posicion.x -= 5.0;
            }
        } else if o.movimiento != Movimiento::Ninguno
            && o.cuerpo.bottom() + 10 < cuerpo_nino.top()
            && !colision_inferior_nino
        {
            let nino = &mut objetos[ultimo];
            nino.posicion.y -= 5.0;
            nino.fuerza_gravedad = 0.0;
        }
    }
}
