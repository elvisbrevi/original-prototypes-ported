// Port de Engine.Datos.Escenario: lee un mapa XML del editor del motor, actualiza la física y los
// objetos y los dibuja por capas (fondo, medio y frente). Las funciones del editor (guardar el
// escenario, la grilla, los datos de depuración) no se usan en el juego y no se portaron.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::audio::Banco;
use crate::motor::fisica;
use crate::motor::objeto::{Colision, Estado, Lugar, Mezcla, Objeto, Tipo, Variacion};
use crate::motor::shader::Materiales;
use crate::motor::{Globales, Post, Shader};

#[derive(Default)]
pub struct Escenario {
    pub nombre: String,
    pub objetos: Vec<Objeto>,
}

/// float.Parse con la cultura del autor (es-CL): la coma es el separador decimal.
fn real(texto: &str) -> f32 {
    texto
        .replace(',', ".")
        .parse()
        .unwrap_or_else(|_| panic!("número {texto:?}"))
}

fn entero(texto: &str) -> i32 {
    texto.parse().unwrap_or_else(|_| panic!("entero {texto:?}"))
}

fn byte(texto: &str) -> u8 {
    texto.parse().unwrap_or_else(|_| panic!("byte {texto:?}"))
}

fn booleano(texto: &str) -> bool {
    match texto {
        "True" => true,
        "False" => false,
        otro => panic!("booleano {otro:?}"),
    }
}

/// Los atributos de un nodo en orden: el motor lee algunos por posición.
fn atributos<'a>(nodo: roxmltree::Node<'a, 'a>) -> Vec<&'a str> {
    nodo.attributes().map(|a| a.value()).collect()
}

impl Escenario {
    /// Escenario.retornarObjetos + cargarObjetos (con Objeto.cargarSonidos).
    pub fn cargar(&mut self, xml: &str, glob: &mut Globales, assets: &Assets, banco: &mut Banco) {
        let documento = roxmltree::Document::parse(xml).expect("mapa XML");
        let raiz = documento.root_element();
        let a = atributos(raiz);
        self.nombre = a[0].to_owned();
        glob.color = [byte(a[1]), byte(a[2]), byte(a[3]), byte(a[4])];
        // a[5..8]: MostrarGrilla, MostrarDatos, MostrarCuerpos (siempre False en los mapas).
        glob.shader = Shader::desde_nombre(a[8]);
        glob.post = Post::nuevo(glob.shader);
        glob.pantalla.posicion = vec2(entero(a[9]) as f32, entero(a[10]) as f32);
        glob.pantalla.tamano = real(a[11]);
        glob.pantalla.inclinacion = real(a[12]);
        glob.ancho = entero(a[13]);
        glob.alto = entero(a[14]);

        banco.reiniciar();
        self.objetos = raiz
            .children()
            .filter(|n| n.is_element())
            .map(|nodo| leer_objeto(nodo, banco))
            .collect();
        for objeto in &mut self.objetos {
            objeto.cargar(assets);
        }
    }

    /// Escenario.actualizar.
    pub fn actualizar(&mut self, glob: &Globales) {
        if glob.pausa {
            return;
        }
        for o in &mut self.objetos {
            o.colision_superior = false;
            o.colision_inferior = false;
            o.colision_derecha = false;
            o.colision_izquierda = false;
        }
        for i in 0..self.objetos.len() {
            let o = &self.objetos[i];
            // Los sistemas de partículas se actualizan aunque no estén activos.
            if o.activo || o.tipo == Tipo::Particulas {
                fisica::aplicar_fisicas(&mut self.objetos, i);
                let o = &mut self.objetos[i];
                o.actualizar(glob);
                o.actualizar_sonidos();
            }
        }
    }

    /// Escenario.dibujarObjetos: borra con el color del mapa y dibuja por capas. Las partículas se
    /// dibujan aunque el sistema no esté activo (las que quedan terminan su vida).
    pub fn dibujar(
        &self,
        glob: &mut Globales,
        assets: &Assets,
        pantalla: &museo::Pantalla,
        materiales: &Materiales,
    ) {
        pantalla.vista(Vec2::ZERO);
        let [r, g, b, a] = glob.color;
        clear_background(Color::from_rgba(r, g, b, a));
        for lugar in [Lugar::Fondo, Lugar::Medio, Lugar::Frente] {
            for o in &self.objetos {
                if o.lugar == lugar && (o.tipo == Tipo::Particulas || o.activo) {
                    o.dibujar(glob, assets, pantalla, materiales);
                }
            }
        }
    }

    /// Escenario.getObjetoPorNombre: el primer objeto cuyo nombre **contiene** el texto (así
    /// "bala1" encuentra "bala1" antes que "bala10" porque está antes en el mapa).
    pub fn indice(&self, nombre: &str) -> usize {
        self.objetos
            .iter()
            .position(|o| o.nombre.contains(nombre))
            .unwrap_or_else(|| panic!("no hay objeto {nombre:?} en {}", self.nombre))
    }

    pub fn objeto(&mut self, nombre: &str) -> &mut Objeto {
        let i = self.indice(nombre);
        &mut self.objetos[i]
    }

    /// Escenario.detectarColision.
    pub fn detectar_colision(&self, a: &str, b: &str) -> bool {
        let ca = self.objetos[self.indice(a)].cuerpo;
        let cb = self.objetos[self.indice(b)].cuerpo;
        ca.intersects(&cb)
    }

    /// Escenario.respawn: tras `tiempo` segundos el objeto reaparece en `posicion`.
    pub fn respawn(&mut self, nombre: &str, posicion: Vec2, tiempo: f32) {
        let o = self.objeto(nombre);
        if o.respawn < tiempo {
            o.respawn += xna::SEGUNDOS_POR_PASO;
            return;
        }
        o.respawn = 0.0;
        o.posicion = posicion;
        o.cuerpo.x = posicion.x as i32;
        o.cuerpo.y = posicion.y as i32;
        o.activo = true;
    }
}

/// Un objeto del mapa: atributos por nombre y los nodos Shader, Estados y Sonidos.
fn leer_objeto(nodo: roxmltree::Node, banco: &mut Banco) -> Objeto {
    let mut o = Objeto::default();
    for atributo in nodo.attributes() {
        let v = atributo.value();
        match atributo.name() {
            "Tipo" => {
                o.tipo = match v {
                    "Grafico" => Tipo::Grafico,
                    "Plataforma" => Tipo::Plataforma,
                    "Personaje" => Tipo::Personaje,
                    "Particulas" => Tipo::Particulas,
                    "Texto" => Tipo::Texto,
                    otro => panic!("tipo {otro}"),
                }
            }
            "Nombre" => o.nombre = v.to_owned(),
            "Texto" => o.texto = v.to_owned(),
            "Nombre_Textura" => o.nombre_textura = v.to_owned(),
            "PosicionX" => o.posicion.x = real(v),
            "PosicionY" => o.posicion.y = real(v),
            "DireccionX" => o.direccion.x = real(v),
            "DireccionY" => o.direccion.y = real(v),
            "Colision" => {
                o.colision = match v {
                    "Inferior" => Colision::Inferior,
                    "Total" => Colision::Total,
                    "Superior" => Colision::Superior,
                    "Ninguna" => Colision::Ninguna,
                    "Lateral" => Colision::Lateral,
                    "Pixel" => Colision::Pixel,
                    _ => o.colision,
                }
            }
            "Velocidad" => o.velocidad = real(v),
            "Inclinacion" => o.inclinacion = real(v),
            "RepetirScroll" => o.repetir_scroll = booleano(v),
            "Activo" => o.activo = booleano(v),
            "NoShaderCamara" => o.no_shader_camara = booleano(v),
            "ColorRandom" => o.color_random = booleano(v),
            "TamañoRandom" => o.tamano_random = booleano(v),
            "DireccionRandom" => o.direccion_random = booleano(v),
            "Gravedad" => o.gravedad = booleano(v),
            "FueraDeCamara" => o.fuera_de_camara = booleano(v),
            "Movimento" => o.movimento = booleano(v),
            "BlendMode" => {
                if v == "AlphaBlend" {
                    o.mezcla = Mezcla::Alfa;
                } else if v == "Additive" {
                    o.mezcla = Mezcla::Aditiva;
                }
            }
            "Animado" => o.animado = booleano(v),
            "ScrollHorizontal" => o.scroll_horizontal = real(v),
            "ScrollVertical" => o.scroll_vertical = real(v),
            "CantidadParticulas" => o.cantidad_particulas = entero(v),
            "Tamaño" => o.tamano = real(v),
            "InicioX" => o.inicio.x = real(v),
            "InicioY" => o.inicio.y = real(v),
            "DestinoX" => o.destino.x = real(v),
            "DestinoY" => o.destino.y = real(v),
            "ColorR" => o.color[0] = byte(v),
            "ColorG" => o.color[1] = byte(v),
            "ColorB" => o.color[2] = byte(v),
            "ColorA" => o.color[3] = byte(v),
            "SpriteEffects" => {
                o.efectos = match v {
                    "FlipHorizontally" => xna::Efectos::HORIZONTAL,
                    "FlipVertically" => xna::Efectos::VERTICAL,
                    "None" => xna::Efectos::NINGUNO,
                    _ => o.efectos,
                }
            }
            "Lugar" => {
                o.lugar = match v {
                    "Fondo" => Lugar::Fondo,
                    "Frente" => Lugar::Frente,
                    "Medio" => Lugar::Medio,
                    _ => o.lugar,
                }
            }
            "CuerpoX" => o.cuerpo.x = real(v) as i32,
            "CuerpoY" => o.cuerpo.y = real(v) as i32,
            "CuerpoAlto" => o.cuerpo.height = real(v) as i32,
            "CuerpoAncho" => o.cuerpo.width = real(v) as i32,
            _ => {}
        }
    }
    for hijo in nodo.children().filter(|n| n.is_element()) {
        if hijo.tag_name().name() == "Shader" {
            let a = atributos(hijo);
            o.shader = Shader::desde_nombre(a[0]);
            o.post = Post::nuevo(o.shader);
            if o.shader == Shader::Brightness
                && let Some(post) = &mut o.post
            {
                post.intensidad = real(a[1]);
            }
        }
    }
    if o.animado {
        for hijo in nodo
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "Estados")
        {
            o.estado_seleccionado = entero(atributos(hijo)[0]) as usize;
            o.estados = hijo
                .children()
                .filter(|n| n.is_element())
                .map(|e| {
                    let a = atributos(e);
                    Estado {
                        cuadro: xna::Rectangle::new(
                            entero(a[1]),
                            entero(a[2]),
                            entero(a[3]),
                            entero(a[4]),
                        ),
                        fps: real(a[5]),
                        numero: real(a[6]),
                        no_repetir: booleano(a[7]),
                        min_variacion: real(a[8]),
                        max_variacion: real(a[9]),
                        velocidad_variacion: real(a[10]),
                        variacion: match a[11] {
                            "Shader" => Variacion::Shader,
                            "Rotacion" => Variacion::Rotacion,
                            "Tamaño" => Variacion::Tamano,
                            _ => Variacion::Ninguna,
                        },
                        ..Default::default()
                    }
                })
                .collect();
        }
    }
    for hijo in nodo
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "Sonidos")
    {
        o.sonidos = hijo
            .children()
            .filter(|n| n.is_element())
            .map(|s| {
                let a = atributos(s);
                banco.tomar(a[0], real(a[1]))
            })
            .collect();
    }
    o
}
