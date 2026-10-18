// Port de Iman.Escenarios: Escenario (base), Escenario_01 y Escenario_02.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::control::Control;
use crate::fisica;
use crate::graficos::{Grafico, Ramas, Tipo};
use crate::objetos::{Colision, Diseno, Movimiento, Objeto};
use crate::xna::{Rectangle, clamp};

/// Iman.Camaras.Camera2D. En el juego original la cámara manual queda desactivada
/// (Initialize pone manualcamera = false) y la escala siempre es 1.
#[derive(Default)]
pub struct Camara {
    pub position: Vec2,
    pub limites: Rectangle,
}

pub struct Escenario {
    pub id: i32,
    pub activo: bool,
    limites: Rectangle,
    objetos: Vec<Objeto>,
    graficos: Vec<Grafico>,
    ramas: Vec<Ramas>,
}

impl Escenario {
    fn vacio(id: i32) -> Self {
        Self {
            id,
            activo: false,
            limites: Rectangle::default(),
            objetos: Vec::new(),
            graficos: Vec::new(),
            ramas: Vec::new(),
        }
    }

    // Escenario_01.cargar
    pub fn escenario_01(assets: &Assets, camara: &mut Camara) -> Self {
        let mut e = Self::vacio(0);
        e.crear_plataforma(
            assets,
            Rectangle::new(0, 510, 600, 25),
            Diseno::Null,
            Colision::Superior,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(550, 400, 200, 150),
            Diseno::Null,
            Colision::Total,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(0, 300, 0, 0),
            Diseno::D02,
            Colision::Superior,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(100, 370, 0, 0),
            Diseno::D02,
            Colision::Superior,
        );
        e.crear_plataforma_iman(assets, vec2(550.0, 300.0), Diseno::D01, Colision::Superior);
        e.crear_plataforma_movediza(
            assets,
            vec2(40.0, 0.0),
            Diseno::D03,
            Movimiento::Vertical,
            220,
            1,
        );
        e.crear_grafico(Tipo::Cielo, assets.cielos[0].clone(), Vec2::ZERO, 1.0);
        e.crear_grafico(
            Tipo::Background,
            assets.backgrounds[0].clone(),
            vec2(0.0, 250.0),
            1.1,
        );
        e.crear_grafico(Tipo::Suelo, assets.suelos[0].clone(), vec2(0.0, 0.0), 1.2);
        e.crear_grafico(Tipo::Front, assets.fronts[0].clone(), vec2(0.0, 550.0), 1.5);
        // SpriteEffects.FlipVertically y SpriteEffects.FlipHorizontally.
        e.ramas.push(Ramas::nueva(
            assets.ramas.clone(),
            vec2(700.0, 270.0),
            0.5,
            false,
            true,
        ));
        e.ramas.push(Ramas::nueva(
            assets.ramas.clone(),
            vec2(700.0, 270.0),
            -0.1,
            true,
            false,
        ));
        e.crear_nino(assets, vec2(50.0, 390.0));
        e.limites = Rectangle::new(0, -100, 750, 480);
        camara.position = vec2(0.0, 150.0);
        camara.limites = Rectangle::new(0, 0, 128, 95);
        e
    }

    // Escenario_02.cargar
    pub fn escenario_02(assets: &Assets, camara: &mut Camara) -> Self {
        let mut e = Self::vacio(1);
        e.crear_plataforma(
            assets,
            Rectangle::new(0, 785, 600, 25),
            Diseno::Null,
            Colision::Superior,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(600, 600, 200, 400),
            Diseno::Null,
            Colision::Total,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(0, 520, 265, 25),
            Diseno::Null,
            Colision::Superior,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(190, 305, 270, 25),
            Diseno::Null,
            Colision::Superior,
        );
        e.crear_plataforma(
            assets,
            Rectangle::new(580, 225, 240, 25),
            Diseno::Null,
            Colision::Superior,
        );
        e.crear_plataforma_iman(assets, vec2(300.0, 500.0), Diseno::D04, Colision::Superior);
        e.crear_plataforma_movediza(
            assets,
            vec2(505.0, 330.0),
            Diseno::D03,
            Movimiento::Vertical,
            50,
            1,
        );
        e.crear_plataforma_movediza(
            assets,
            vec2(300.0, 320.0),
            Diseno::D03,
            Movimiento::Vertical,
            70,
            2,
        );
        e.crear_plataforma_movediza(
            assets,
            vec2(35.0, 120.0),
            Diseno::D03,
            Movimiento::Vertical,
            200,
            2,
        );
        e.crear_grafico(Tipo::Cielo, assets.cielos[1].clone(), Vec2::ZERO, 1.1);
        e.crear_grafico(
            Tipo::Background,
            assets.backgrounds[1].clone(),
            vec2(0.0, 300.0),
            1.1,
        );
        e.crear_grafico(Tipo::Suelo, assets.suelos[1].clone(), vec2(0.0, 120.0), 1.2);
        e.crear_grafico(
            Tipo::Front,
            assets.fronts[1].clone(),
            vec2(400.0, 1000.0),
            0.9,
        );
        e.crear_grafico(
            Tipo::Front,
            assets.fronts[1].clone(),
            vec2(0.0, 1040.0),
            0.9,
        );
        e.crear_nino(assets, vec2(50.0, 680.0));
        e.limites = Rectangle::new(0, -100, 750, 480);
        camara.position = vec2(0.0, 150.0);
        camara.limites = Rectangle::new(0, -200, 128, 600);
        e
    }

    pub fn actualizar(&mut self, camara: &mut Camara, control: &Control, assets: &Assets) {
        fisica::actualizar(&mut self.objetos);
        self.mover_camara(camara);
        for grafico in &mut self.graficos {
            grafico.actualizar(camara.position);
        }
        for rama in &mut self.ramas {
            rama.actualizar();
        }
        for objeto in &mut self.objetos {
            objeto.actualizar(control, assets);
        }
        let nino = self.nino().posicion;
        let terminado = match self.id {
            0 => nino.y < 20.0,
            _ => nino.y < 150.0 && nino.x > 660.0,
        };
        if terminado {
            self.activo = false;
        }
    }

    pub fn dibujar(&self, camara: &Camara) {
        let cam = camara.position;
        let g = &self.graficos;
        match self.id {
            0 => {
                g[0].dibujar(cam);
                g[1].dibujar(cam);
                self.ramas[0].dibujar();
                g[2].dibujar(cam);
                self.ramas[1].dibujar();
                self.dibujar_objetos();
                g[3].dibujar(cam);
            }
            _ => {
                g[0].dibujar(cam);
                g[1].dibujar(cam);
                g[2].dibujar(cam);
                self.dibujar_objetos();
                g[3].dibujar(cam);
                g[4].dibujar(cam);
            }
        }
    }

    fn dibujar_objetos(&self) {
        for objeto in self.objetos.iter().filter(|o| o.diseno != Diseno::Null) {
            objeto.dibujar();
        }
    }

    /// El niño siempre es el último objeto creado.
    fn nino(&self) -> &Objeto {
        self.objetos.last().expect("el escenario no tiene niño")
    }

    fn crear_plataforma(
        &mut self,
        assets: &Assets,
        rectangulo: Rectangle,
        diseno: Diseno,
        colision: Colision,
    ) {
        self.objetos.push(Objeto::plataforma(assets, diseno, |p| {
            p.cuerpo = rectangulo;
            p.colision = colision;
            p.posicion = vec2(rectangulo.x as f32, rectangulo.y as f32);
        }));
    }

    fn crear_plataforma_iman(
        &mut self,
        assets: &Assets,
        posicion: Vec2,
        diseno: Diseno,
        colision: Colision,
    ) {
        self.objetos.push(Objeto::plataforma(assets, diseno, |p| {
            p.colision = colision;
            p.posicion = posicion;
            p.gravedad = true;
        }));
    }

    fn crear_plataforma_movediza(
        &mut self,
        assets: &Assets,
        posicion: Vec2,
        diseno: Diseno,
        movimiento: Movimiento,
        rango_movimiento: i32,
        velocidad: i32,
    ) {
        self.objetos.push(Objeto::plataforma(assets, diseno, |p| {
            p.colision = Colision::Superior;
            p.posicion = posicion;
            p.movimiento = movimiento;
            p.velocidad = velocidad;
            let inicio = if movimiento == Movimiento::Horizontal {
                posicion.x
            } else {
                posicion.y
            } as i32;
            p.posicion_inicial = inicio;
            p.posicion_final = inicio + rango_movimiento;
        }));
    }

    fn crear_grafico(&mut self, tipo: Tipo, textura: Texture2D, posicion: Vec2, zoom: f32) {
        self.graficos
            .push(Grafico::nuevo(tipo, textura, posicion, zoom));
    }

    fn crear_nino(&mut self, assets: &Assets, posicion: Vec2) {
        self.objetos.push(Objeto::nino(assets, posicion));
    }

    fn mover_camara(&mut self, camara: &mut Camara) {
        let limites = self.limites;
        let nino = self.objetos.last_mut().expect("el escenario no tiene niño");
        let l = camara.limites;
        camara.position.x = clamp(nino.origen.x - 320.0, l.left() as f32, l.right() as f32);
        camara.position.y = clamp(nino.origen.y - 240.0, l.top() as f32, l.bottom() as f32);
        nino.posicion.x = clamp(
            nino.posicion.x,
            limites.left() as f32,
            (limites.right() - nino.cuerpo.width) as f32,
        );
    }
}
