// La escena de un juego 3D de Unity 3.x: GameObjects con su Transform (jerarquía con posición,
// rotación y escala locales) y los componentes que usa el juego (MeshRenderer, colisionadores,
// Rigidbody, Light, Camera, AudioSource, GUITexture, partículas heredadas y MonoBehaviour). Destroy
// marca el objeto y sus hijos, que se borran al final del frame, como en Unity.

use std::collections::HashMap;

use macroquad::prelude::{Color, Mat4, Quat, Vec3};
use serde_json::Value;

use crate::datos3d::{Componente3d, DatosObjeto3d, Proyecto3d};
use crate::escena::Transformacion;
use crate::fisica3d::{Colisionador3d, Forma3d, Rigidbody3d};
use crate::particulas3d::SistemaLegado;

pub type Id = usize;

/// MeshRenderer.
pub struct RendererMalla {
    pub activo: bool,
    pub materiales: Vec<Option<String>>,
}

/// Un MonoBehaviour: el juego traduce cada script y lee aquí sus campos serializados.
pub struct Script {
    pub nombre: String,
    pub activo: bool,
    pub campos: Value,
}

/// AudioSource (con lo que hace falta para la atenuación 3D).
#[derive(Clone)]
pub struct FuenteAudio3d {
    pub activo: bool,
    pub clip: Option<String>,
    pub al_despertar: bool,
    pub volumen: f32,
    pub bucle: bool,
    pub mudo: bool,
    pub distancia_minima: f32,
    pub distancia_maxima: f32,
    pub atenuacion: u32,
}

/// Light.
#[derive(Clone)]
pub struct Luz {
    pub activo: bool,
    pub clase: u32,
    pub color: Color,
    pub intensidad: f32,
    pub rango: f32,
    pub destello: Option<String>,
}

/// Camera.
#[derive(Clone)]
pub struct DatosCamara3d {
    pub activo: bool,
    pub borrado: u32,
    pub fondo: Color,
    pub campo_vision: f32,
    pub cerca: f32,
    pub lejos: f32,
}

/// GUITexture.
pub struct TexturaGui {
    pub activo: bool,
    pub textura: Option<String>,
    pub color: Color,
    pub margen: [f32; 4],
}

pub struct Objeto3d {
    /// El id del GameObject en el archivo original.
    pub original: i64,
    pub nombre: String,
    pub etiqueta: String,
    pub capa: u32,
    pub activo: bool,
    pub local: Transformacion,
    pub padre: Option<Id>,
    pub hijos: Vec<Id>,
    /// MeshFilter.
    pub malla: Option<String>,
    pub renderer: Option<RendererMalla>,
    pub colisionadores: Vec<Colisionador3d>,
    pub cuerpo: Option<Rigidbody3d>,
    pub luz: Option<Luz>,
    pub camara: Option<DatosCamara3d>,
    pub audio: Vec<FuenteAudio3d>,
    pub textura_gui: Option<TexturaGui>,
    pub particulas: Option<SistemaLegado>,
    pub scripts: Vec<Script>,
    /// Un Terrain (el componente con sus datos va aparte).
    pub terreno: Option<Value>,
    pub destruido: bool,
}

impl Objeto3d {
    pub fn script(&self, nombre: &str) -> Option<&Script> {
        self.scripts.iter().find(|s| s.nombre == nombre)
    }
}

fn color(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

pub struct Escena3d {
    /// El índice de la escena en BuildSettings (Application.loadedLevel).
    pub nivel: usize,
    pub nombre: String,
    objetos: Vec<Option<Objeto3d>>,
    originales: HashMap<i64, Id>,
}

impl Escena3d {
    /// Application.LoadLevel(nivel): los objetos de la escena en el orden del archivo.
    pub fn cargar(proyecto: &Proyecto3d, nivel: usize) -> Self {
        let datos = &proyecto.escenas[nivel];
        let mut escena = Self {
            nivel,
            nombre: datos.nombre.clone(),
            objetos: Vec::new(),
            originales: HashMap::new(),
        };
        for d in &datos.objetos {
            escena.originales.insert(d.id, escena.objetos.len());
            escena.objetos.push(Some(Self::objeto_de(d)));
        }
        for d in &datos.objetos {
            let id = escena.originales[&d.id];
            let padre = d.padre.and_then(|p| escena.originales.get(&p).copied());
            let hijos = d
                .hijos
                .iter()
                .filter_map(|h| escena.originales.get(h).copied())
                .collect();
            let obj = escena.objeto_mut(id);
            obj.padre = padre;
            obj.hijos = hijos;
        }
        escena
    }

    fn objeto_de(d: &DatosObjeto3d) -> Objeto3d {
        let mut obj = Objeto3d {
            original: d.id,
            nombre: d.nombre.clone(),
            etiqueta: d.etiqueta.clone(),
            capa: d.capa,
            activo: d.activo,
            local: Transformacion {
                posicion: Vec3::from(d.posicion),
                rotacion: Quat::from_array(d.rotacion),
                escala: Vec3::from(d.escala),
            },
            padre: None,
            hijos: Vec::new(),
            malla: None,
            renderer: None,
            colisionadores: Vec::new(),
            cuerpo: None,
            luz: None,
            camara: None,
            audio: Vec::new(),
            textura_gui: None,
            particulas: None,
            scripts: Vec::new(),
            terreno: None,
            destruido: false,
        };
        let mut emisor = None;
        let mut animador = None;
        let mut renderer_particulas = None;
        let mut choque_particulas = None;
        for c in &d.componentes {
            match c {
                Componente3d::MeshFilter { malla } => obj.malla = malla.clone(),
                Componente3d::MeshRenderer { activo, materiales } => {
                    obj.renderer = Some(RendererMalla {
                        activo: *activo,
                        materiales: materiales.clone(),
                    })
                }
                Componente3d::MeshCollider {
                    activo,
                    trigger,
                    malla,
                    ..
                } => {
                    if let Some(m) = malla {
                        obj.colisionadores.push(Colisionador3d::nuevo(
                            *activo,
                            *trigger,
                            Forma3d::Malla(m.clone()),
                        ))
                    }
                }
                Componente3d::BoxCollider {
                    activo,
                    trigger,
                    centro,
                    tamano,
                } => obj.colisionadores.push(Colisionador3d::nuevo(
                    *activo,
                    *trigger,
                    Forma3d::Caja {
                        centro: Vec3::from(*centro),
                        tamano: Vec3::from(*tamano),
                    },
                )),
                Componente3d::SphereCollider {
                    activo,
                    trigger,
                    centro,
                    radio,
                } => obj.colisionadores.push(Colisionador3d::nuevo(
                    *activo,
                    *trigger,
                    Forma3d::Esfera {
                        centro: Vec3::from(*centro),
                        radio: *radio,
                    },
                )),
                Componente3d::TerrainCollider { activo } => obj
                    .colisionadores
                    .push(Colisionador3d::nuevo(*activo, false, Forma3d::Terreno)),
                Componente3d::Rigidbody {
                    masa,
                    arrastre,
                    arrastre_angular,
                    gravedad,
                    cinematico,
                    restricciones,
                } => {
                    obj.cuerpo = Some(Rigidbody3d::nuevo(
                        *masa,
                        *arrastre,
                        *arrastre_angular,
                        *gravedad,
                        *cinematico,
                        *restricciones,
                    ))
                }
                Componente3d::Light {
                    activo,
                    clase,
                    color: c,
                    intensidad,
                    rango,
                    destello,
                } => {
                    obj.luz = Some(Luz {
                        activo: *activo,
                        clase: *clase,
                        color: color(*c),
                        intensidad: *intensidad,
                        rango: *rango,
                        destello: destello.clone(),
                    })
                }
                Componente3d::Camera {
                    activo,
                    borrado,
                    fondo,
                    campo_vision,
                    cerca,
                    lejos,
                } => {
                    obj.camara = Some(DatosCamara3d {
                        activo: *activo,
                        borrado: *borrado,
                        fondo: color(*fondo),
                        campo_vision: *campo_vision,
                        cerca: *cerca,
                        lejos: *lejos,
                    })
                }
                Componente3d::AudioSource {
                    activo,
                    clip,
                    al_despertar,
                    volumen,
                    bucle,
                    mudo,
                    distancia_minima,
                    distancia_maxima,
                    atenuacion,
                } => obj.audio.push(FuenteAudio3d {
                    activo: *activo,
                    clip: clip.clone(),
                    al_despertar: *al_despertar,
                    volumen: *volumen,
                    bucle: *bucle,
                    mudo: *mudo,
                    distancia_minima: *distancia_minima,
                    distancia_maxima: *distancia_maxima,
                    atenuacion: *atenuacion,
                }),
                Componente3d::GUITexture {
                    activo,
                    textura,
                    color: c,
                    margen,
                } => {
                    obj.textura_gui = Some(TexturaGui {
                        activo: *activo,
                        textura: textura.clone(),
                        color: color(*c),
                        margen: *margen,
                    })
                }
                Componente3d::EllipsoidParticleEmitter(e) => emisor = Some(e.as_ref().clone()),
                Componente3d::ParticleAnimator(a) => animador = Some(a.as_ref().clone()),
                Componente3d::ParticleRenderer {
                    activo,
                    materiales,
                    tamano_maximo,
                    ..
                } => {
                    renderer_particulas = Some((
                        *activo,
                        materiales.first().cloned().flatten(),
                        *tamano_maximo,
                    ))
                }
                Componente3d::WorldParticleCollider {
                    rebote,
                    perdida,
                    mensaje,
                    velocidad_minima,
                    ..
                } => choque_particulas = Some((*rebote, *perdida, *mensaje, *velocidad_minima)),
                Componente3d::MonoBehaviour {
                    activo,
                    script,
                    campos,
                } => {
                    if script == "Terrain" {
                        obj.terreno = Some(campos.clone());
                    } else {
                        obj.scripts.push(Script {
                            nombre: script.clone(),
                            activo: *activo,
                            campos: campos.clone(),
                        })
                    }
                }
                Componente3d::Animation { .. } | Componente3d::Otro => {}
            }
        }
        if let Some(e) = emisor {
            obj.particulas = Some(SistemaLegado::nuevo(
                e,
                animador,
                renderer_particulas,
                choque_particulas,
            ));
        }
        obj
    }

    /// El objeto de la escena con este id del archivo original.
    pub fn original(&self, id: i64) -> Option<Id> {
        self.originales
            .get(&id)
            .copied()
            .filter(|&i| self.existe(i))
    }

    pub fn existe(&self, id: Id) -> bool {
        self.objetos.get(id).is_some_and(|o| o.is_some())
    }

    pub fn objeto(&self, id: Id) -> &Objeto3d {
        self.objetos[id].as_ref().expect("objeto vivo")
    }

    pub fn objeto_mut(&mut self, id: Id) -> &mut Objeto3d {
        self.objetos[id].as_mut().expect("objeto vivo")
    }

    /// Los objetos vivos, en el orden del archivo.
    pub fn ids(&self) -> Vec<Id> {
        (0..self.objetos.len())
            .filter(|&i| self.objetos[i].is_some())
            .collect()
    }

    /// Si el objeto y sus padres están activos (en Go Ball todos lo están).
    pub fn activo_en_jerarquia(&self, id: Id) -> bool {
        let mut actual = Some(id);
        while let Some(i) = actual {
            let o = self.objeto(i);
            if !o.activo {
                return false;
            }
            actual = o.padre;
        }
        true
    }

    /// GameObject.Find.
    pub fn buscar(&self, nombre: &str) -> Option<Id> {
        self.ids()
            .into_iter()
            .find(|&i| self.objeto(i).nombre == nombre && self.activo_en_jerarquia(i))
    }

    /// localToWorldMatrix.
    pub fn matriz_mundo(&self, id: Id) -> Mat4 {
        let o = self.objeto(id);
        let local = o.local.matriz();
        match o.padre {
            Some(p) => self.matriz_mundo(p) * local,
            None => local,
        }
    }

    /// transform.position.
    pub fn posicion(&self, id: Id) -> Vec3 {
        self.matriz_mundo(id).transform_point3(Vec3::ZERO)
    }

    /// transform.rotation.
    pub fn rotacion(&self, id: Id) -> Quat {
        let o = self.objeto(id);
        match o.padre {
            Some(p) => self.rotacion(p) * o.local.rotacion,
            None => o.local.rotacion,
        }
    }

    /// transform.position = p.
    pub fn fijar_posicion(&mut self, id: Id, p: Vec3) {
        let local = match self.objeto(id).padre {
            Some(padre) => self.matriz_mundo(padre).inverse().transform_point3(p),
            None => p,
        };
        self.objeto_mut(id).local.posicion = local;
    }

    /// transform.rotation = r.
    pub fn fijar_rotacion(&mut self, id: Id, r: Quat) {
        let local = match self.objeto(id).padre {
            Some(padre) => self.rotacion(padre).inverse() * r,
            None => r,
        };
        self.objeto_mut(id).local.rotacion = local;
    }

    /// transform.Rotate(euler, Space.Self): gira en los ejes locales, en el orden z, x, y de
    /// Quaternion.Euler.
    pub fn girar_local(&mut self, id: Id, grados: Vec3) {
        let o = self.objeto_mut(id);
        o.local.rotacion = (o.local.rotacion * euler(grados)).normalize();
    }

    /// Object.Destroy: el objeto y sus hijos se borran al final del frame.
    pub fn destruir(&mut self, id: Id) {
        if !self.existe(id) {
            return;
        }
        self.objeto_mut(id).destruido = true;
        for h in self.objeto(id).hijos.clone() {
            self.destruir(h);
        }
    }

    /// Borra los objetos destruidos durante el frame.
    pub fn limpiar_destruidos(&mut self) {
        for id in self.ids() {
            if !self.objeto(id).destruido {
                continue;
            }
            if let Some(p) = self.objeto(id).padre
                && self.existe(p)
            {
                self.objeto_mut(p).hijos.retain(|&h| h != id);
            }
            self.objetos[id] = None;
        }
    }
}

/// Quaternion.Euler(x, y, z) en grados: primero z, luego x y luego y (en los ejes del mundo).
pub fn euler(g: Vec3) -> Quat {
    let r = g * (std::f32::consts::PI / 180.0);
    Quat::from_rotation_y(r.y) * Quat::from_rotation_x(r.x) * Quat::from_rotation_z(r.z)
}

/// Transform.eulerAngles: los ángulos (en grados, de 0 a 360) que devuelven `q` con euler().
pub fn angulos_euler(q: Quat) -> Vec3 {
    let (y, x, z) = q.to_euler(macroquad::math::EulerRot::YXZ);
    let g = |a: f32| {
        let d = a.to_degrees();
        if d < 0.0 { d + 360.0 } else { d }
    };
    Vec3::new(g(x), g(y), g(z))
}
