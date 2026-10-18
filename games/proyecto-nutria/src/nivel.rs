// Port de Proyecto_Nutria.Nivel y Nivel_1: el único nivel del prototipo.

use macroquad::prelude::*;
use xna::Rectangle;

use crate::assets::Assets;
use crate::caja::Caja;
use crate::camara::Camara;
use crate::control::Control;
use crate::fisica::Fisica;
use crate::iconos::IconoPersonajes;
use crate::menu::MenuPersonajes;
use crate::personaje::Personaje;
use crate::plataforma::{Colision, Diseno, Movimiento, Plataforma};
use crate::shader::BlurGrayscale;
use crate::tiempo::GameTime;

pub struct Nivel {
    pub efecto: BlurGrayscale,
    pub icon_per: Vec<IconoPersonajes>,
    pub limites: Rectangle,
    pub personajes: Vec<Personaje>,
    pub menu: MenuPersonajes,
    pub cajas: Vec<Caja>,
    pub plataformas: Vec<Plataforma>,
    pub fisica: Fisica,
}

/// Nivel.crearPlataforma: invisible (diseño Null), con la colisión indicada.
fn fija(cuerpo: Rectangle, colision: Colision) -> Plataforma {
    let mut p = Plataforma {
        posicion: vec2(cuerpo.x as f32, cuerpo.y as f32),
        cuerpo,
        colision,
        ..Default::default()
    };
    p.cargar();
    p
}

/// Nivel.crearPlataformaMovediza: de roca, solo choca por arriba, a 3 px por paso. Su cuerpo se
/// calcula al actualizarla.
fn movediza(posicion: Vec2, max: f32, min: f32, movimiento: Movimiento) -> Plataforma {
    let mut p = Plataforma {
        posicion,
        colision: Colision::Superior,
        diseno: Diseno::Roca,
        movimiento,
        posicion_inicial: min,
        posicion_final: max,
        velocidad_movimiento: 3.0,
        ..Default::default()
    };
    p.cargar();
    p
}

/// Nivel.crearSubida: una rampa de plataformas invisibles de 1x1 con colisión total.
fn subida(inicio: Vec2, cantidad: usize) -> Vec<Plataforma> {
    let mut posicion = inicio;
    (0..cantidad)
        .map(|_| {
            let mut p = Plataforma {
                posicion,
                cuerpo: Rectangle::new(posicion.x as i32, posicion.y as i32, 1, 1),
                colision: Colision::Total,
                ..Default::default()
            };
            posicion += vec2(1.0, -1.0);
            p.cargar();
            p
        })
        .collect()
}

impl Nivel {
    /// Nivel_1.cargar. Crea un coipo y un pollo (y por error coloca dos veces al coipo), pero
    /// MenuPersonajes.cargar los reemplaza enseguida, así que aquí solo están los de ese menú.
    pub fn cargar_nivel_1(assets: &Assets) -> Self {
        let limites = Rectangle::new(50, 0, 2621, 1350);
        let cajas = [
            (1500.0, 270.0),
            (500.0, 270.0),
            (2500.0, 270.0),
            (2700.0, 490.0),
            (1100.0, 490.0),
            (100.0, 710.0),
            (1000.0, 710.0),
            (2100.0, 710.0),
            (2700.0, 820.0),
        ]
        .map(|(x, y)| Caja::nueva(vec2(x, y), assets));

        // Mismo orden que Nivel_1.cargar: el orden decide qué colisión se resuelve primero.
        let r = Rectangle::new;
        let mut plataformas = vec![
            fija(r(-50, 0, 50, 1500), Colision::Lateral),
            fija(r(2850, 0, 50, 1500), Colision::Lateral),
            fija(r(30, 350, 620, 30), Colision::Superior),
            fija(r(900, 350, 1130, 30), Colision::Superior),
            fija(r(2230, 350, 650, 30), Colision::Superior),
            fija(r(580, 460, 430, 30), Colision::Superior),
            movediza(vec2(2030.0, 460.0), 650.0, 370.0, Movimiento::Vertical),
            fija(r(2420, 460, 430, 30), Colision::Superior),
            fija(r(2420, 570, 430, 30), Colision::Superior),
            fija(r(230, 570, 250, 30), Colision::Superior),
        ];
        plataformas.extend(subida(vec2(470.0, 570.0), 110));
        plataformas.extend([
            fija(r(1020, 570, 330, 30), Colision::Superior),
            movediza(vec2(0.0, 680.0), 1000.0, 0.0, Movimiento::Horizontal),
            fija(r(1340, 680, 440, 30), Colision::Superior),
            fija(r(2250, 680, 600, 30), Colision::Superior),
            fija(r(0, 790, 330, 30), Colision::Superior),
            fija(r(570, 790, 650, 30), Colision::Superior),
            fija(r(1800, 790, 450, 30), Colision::Superior),
            fija(r(0, 900, 2900, 30), Colision::Superior),
        ]);
        plataformas.extend(subida(vec2(1700.0, 900.0), 110));

        let (menu, personajes, icon_per) = MenuPersonajes::cargar(limites, assets);
        Self {
            efecto: BlurGrayscale::cargar(),
            icon_per,
            limites,
            personajes,
            menu,
            cajas: cajas.into(),
            plataformas,
            fisica: Fisica::default(),
        }
    }

    /// Nivel_1.actualizar. Solo los personajes con control (uno, con el teclado) se actualizan,
    /// y solo sus iconos; el pollo sigue sometido a la física. (El cronómetro, invisible, no se
    /// portó.)
    pub fn actualizar(
        &mut self,
        tiempo: &GameTime,
        controles: &[Control],
        camara: &mut Camara,
        assets: &Assets,
    ) {
        if !self.menu.activo {
            camara.activo = true;
            for plataforma in &mut self.plataformas {
                if plataforma.diseno != Diseno::Null {
                    plataforma.actualizar(assets);
                }
            }
            for (personaje, control) in self.personajes.iter_mut().zip(controles) {
                personaje.actualizar(tiempo, control, assets);
            }
            for caja in &mut self.cajas {
                caja.actualizar();
            }
            self.fisica
                .actualizar(&mut self.personajes, &self.plataformas, &mut self.cajas);
            for (icono, personaje) in self
                .icon_per
                .iter_mut()
                .zip(&self.personajes)
                .take(controles.len())
            {
                icono.actualizar(personaje);
            }
        }
        self.menu.actualizar(
            &controles[0],
            &mut self.personajes,
            &mut self.icon_per,
            self.limites,
            camara,
            assets,
        );
        self.efecto.activo = self.menu.activo;
    }

    /// Nivel_1.dibujar, dentro del área de juego. `vista` fija la cámara del área: con Some((posición,
    /// escala)) la del mundo, con None la de pantalla (sin transformación).
    pub fn dibujar(&self, camara: &Camara, assets: &Assets, vista: impl Fn(Option<(Vec2, f32)>)) {
        vista(None);
        draw_texture(&assets.cielo, 0.0, 0.0, WHITE);
        vista(Some(camara.transform));
        self.efecto.inicio();
        let cam = camara.posicion;
        let capa = |textura: &Texture2D, posicion: Vec2, alto_extra: i32, escala: f32| {
            let fuente = Rectangle::new(
                0,
                0,
                textura.width() as i32,
                textura.height() as i32 + alto_extra,
            );
            xna::dibujar(
                textura,
                posicion,
                Some(fuente),
                WHITE,
                0.0,
                Vec2::ZERO,
                escala,
                false,
            );
        };
        capa(
            &assets.scroll2,
            vec2((cam.x - 250.0) * 0.7, (cam.y - 500.0) * 0.7),
            0,
            4.0,
        );
        // El recorte de scroll1 es 150 px más alto que la textura: el modo Clamp estira su
        // última fila.
        capa(
            &assets.scroll1,
            vec2((cam.x - 100.0) * 0.5, (cam.y - 600.0) * 0.5),
            150,
            2.9,
        );
        xna::dibujar(
            &assets.diseno_trasero,
            Vec2::ZERO,
            Some(Rectangle::new(0, 0, 1280, 620)),
            WHITE,
            0.0,
            Vec2::ZERO,
            2.25,
            false,
        );
        // menu.PersonajesActivos siempre es true.
        for personaje in &self.personajes {
            personaje.dibujar(assets);
        }
        for caja in &self.cajas {
            caja.dibujar(assets);
        }
        for plataforma in &self.plataformas {
            if plataforma.diseno != Diseno::Null {
                plataforma.dibujar(assets);
            }
        }
        self.efecto.fin();
        if self.menu.activo {
            vista(None);
            self.menu.dibujar(assets);
        }
    }
}
