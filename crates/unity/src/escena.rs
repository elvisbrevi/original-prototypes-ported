// La escena: GameObjects con su Transform (jerarquía con posición, rotación y escala locales) y
// los componentes que usan los juegos. Instantiate copia la jerarquía de un prefab; Destroy marca
// el objeto y sus hijos, que se borran al final del frame, como en Unity.

use std::collections::HashMap;

use macroquad::prelude::{Color, Mat4, Quat, Vec2, Vec3, vec3};
use serde_json::Value;

use crate::animator::Animator;
use crate::datos::{Componente, DatosObjeto, Proyecto, ValorAnimado};
use crate::fisica2d::{Colisionador2D, Fisica2d, Forma2D, Rigidbody2D};
use crate::particulas::Particulas;

pub type Id = usize;

#[derive(Clone, Copy, Debug)]
pub struct Transformacion {
    pub posicion: Vec3,
    pub rotacion: Quat,
    pub escala: Vec3,
}

impl Transformacion {
    pub fn matriz(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.escala, self.rotacion, self.posicion)
    }
}

pub struct SpriteRenderer {
    pub activo: bool,
    pub sprite: Option<String>,
    pub color: Color,
    /// Índice de la sorting layer en el orden de dibujo.
    pub capa_orden: usize,
    pub orden: i32,
}

/// Un MonoBehaviour: el juego traduce cada script y lee aquí sus campos serializados.
pub struct Script {
    pub nombre: String,
    pub activo: bool,
    pub campos: Value,
}

pub struct FuenteAudio {
    pub activo: bool,
    pub clip: Option<String>,
    pub bucle: bool,
    pub volumen: f32,
    pub mudo: bool,
    pub al_despertar: bool,
}

/// GUIText: un texto en coordenadas de la vista, dibujado por el GUILayer.
pub struct TextoGui {
    pub activo: bool,
    pub texto: String,
    pub fuente: Option<String>,
    pub tamano: u32,
    pub estilo: u32,
    pub anclaje: u32,
    pub alineacion: u32,
    pub desplazamiento: Vec2,
    pub interlineado: f32,
    pub color: Color,
}

pub struct DatosCamara {
    pub fondo: Color,
    pub ortografica: bool,
    pub tamano_ortografico: f32,
    pub campo_vision: f32,
    pub cerca: f32,
    pub lejos: f32,
}

pub struct Objeto {
    pub nombre: String,
    pub etiqueta: String,
    pub capa: u32,
    pub activo: bool,
    pub local: Transformacion,
    pub padre: Option<Id>,
    pub hijos: Vec<Id>,
    pub sprite: Option<SpriteRenderer>,
    pub colisionadores: Vec<Colisionador2D>,
    pub cuerpo: Option<Rigidbody2D>,
    pub animator: Option<Animator>,
    pub particulas: Option<Particulas>,
    pub audio: Vec<FuenteAudio>,
    pub scripts: Vec<Script>,
    pub camara: Option<DatosCamara>,
    pub texto_gui: Option<TextoGui>,
    pub destruido: bool,
}

impl Objeto {
    pub fn script(&self, nombre: &str) -> Option<&Script> {
        self.scripts.iter().find(|s| s.nombre == nombre)
    }
}

fn color(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

pub struct Escena {
    objetos: Vec<Option<Objeto>>,
    /// Los ids del archivo original de los objetos de la escena cargada.
    originales: HashMap<i64, Id>,
    pub fisica: Fisica2d,
}

impl Escena {
    /// Application.LoadLevel: la primera escena del proyecto, con sus objetos en el orden del
    /// archivo.
    pub fn cargar(proyecto: &Proyecto) -> Self {
        Self::cargar_nivel(proyecto, 0)
    }

    /// Application.LoadLevel(nivel).
    pub fn cargar_nivel(proyecto: &Proyecto, nivel: usize) -> Self {
        let mut escena = Self {
            objetos: Vec::new(),
            originales: HashMap::new(),
            fisica: Fisica2d::nueva(&proyecto.fisica2d),
        };
        let originales = escena.crear(proyecto, proyecto.objetos_de(nivel));
        escena.originales = originales;
        escena
    }

    /// Crea los objetos de una lista (escena o prefab) y devuelve el mapa de ids originales.
    fn crear(&mut self, proyecto: &Proyecto, datos: &[DatosObjeto]) -> HashMap<i64, Id> {
        let mut ids = HashMap::new();
        for d in datos {
            ids.insert(d.id, self.objetos.len());
            self.objetos.push(Some(Self::objeto_de(proyecto, d)));
        }
        for d in datos {
            let id = ids[&d.id];
            let obj = self.objetos[id].as_mut().expect("recién creado");
            obj.padre = d.padre.and_then(|p| ids.get(&p).copied());
            obj.hijos = d.hijos.iter().filter_map(|h| ids.get(h).copied()).collect();
        }
        ids
    }

    fn objeto_de(proyecto: &Proyecto, d: &DatosObjeto) -> Objeto {
        let capa_orden = |nombre: &str| {
            proyecto
                .capas_orden
                .iter()
                .position(|c| c == nombre)
                .unwrap_or(0)
        };
        let material = |nombre: &Option<String>| {
            nombre
                .as_ref()
                .and_then(|m| proyecto.materiales_fisicos.get(m).copied())
        };
        let mut obj = Objeto {
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
            sprite: None,
            colisionadores: Vec::new(),
            cuerpo: None,
            animator: None,
            particulas: None,
            audio: Vec::new(),
            scripts: Vec::new(),
            camara: None,
            texto_gui: None,
            destruido: false,
        };
        let mut renderer_particulas = None;
        let mut sistema = None;
        for c in &d.componentes {
            match c {
                Componente::SpriteRenderer {
                    activo,
                    sprite,
                    color: c,
                    capa_orden: capa,
                    orden,
                } => {
                    obj.sprite = Some(SpriteRenderer {
                        activo: *activo,
                        sprite: sprite.clone(),
                        color: color(*c),
                        capa_orden: capa_orden(capa),
                        orden: *orden,
                    })
                }
                Componente::BoxCollider2D {
                    activo,
                    trigger,
                    centro,
                    material: m,
                    tamano,
                } => obj.colisionadores.push(Colisionador2D::nuevo(
                    *activo,
                    *trigger,
                    Vec2::from(*centro),
                    Forma2D::Caja(Vec2::from(*tamano)),
                    material(m),
                )),
                Componente::CircleCollider2D {
                    activo,
                    trigger,
                    centro,
                    material: m,
                    radio,
                } => obj.colisionadores.push(Colisionador2D::nuevo(
                    *activo,
                    *trigger,
                    Vec2::from(*centro),
                    Forma2D::Circulo(*radio),
                    material(m),
                )),
                Componente::PolygonCollider2D {
                    activo,
                    trigger,
                    centro,
                    material: m,
                    caminos,
                } => obj.colisionadores.push(Colisionador2D::nuevo(
                    *activo,
                    *trigger,
                    Vec2::from(*centro),
                    Forma2D::Poligono(
                        caminos
                            .iter()
                            .map(|c| c.iter().map(|&p| Vec2::from(p)).collect())
                            .collect(),
                    ),
                    material(m),
                )),
                Componente::Rigidbody2D {
                    masa,
                    arrastre,
                    arrastre_angular,
                    escala_gravedad,
                    angulo_fijo,
                    cinematico,
                    sueno,
                    interpolacion,
                } => {
                    obj.cuerpo = Some(Rigidbody2D {
                        masa: *masa,
                        arrastre: *arrastre,
                        arrastre_angular: *arrastre_angular,
                        escala_gravedad: *escala_gravedad,
                        angulo_fijo: *angulo_fijo,
                        cinematico: *cinematico,
                        sueno: *sueno,
                        interpolacion: *interpolacion,
                    })
                }
                Componente::Animator {
                    activo,
                    controlador,
                    modo,
                } => {
                    let mut a = Animator::nuevo(*activo, &proyecto.controladores[controlador]);
                    a.modo = *modo;
                    obj.animator = Some(a)
                }
                Componente::TextoGui {
                    activo,
                    texto,
                    fuente,
                    tamano,
                    estilo,
                    anclaje,
                    alineacion,
                    desplazamiento,
                    interlineado,
                    color: c,
                } => {
                    obj.texto_gui = Some(TextoGui {
                        activo: *activo,
                        texto: texto.clone(),
                        fuente: fuente.clone(),
                        tamano: *tamano,
                        estilo: *estilo,
                        anclaje: *anclaje,
                        alineacion: *alineacion,
                        desplazamiento: Vec2::from(*desplazamiento),
                        interlineado: *interlineado,
                        color: color(*c),
                    })
                }
                Componente::AudioSource {
                    activo,
                    clip,
                    al_despertar,
                    volumen,
                    bucle,
                    mudo,
                } => obj.audio.push(FuenteAudio {
                    activo: *activo,
                    clip: clip.clone(),
                    bucle: *bucle,
                    volumen: *volumen,
                    mudo: *mudo,
                    al_despertar: *al_despertar,
                }),
                Componente::MonoBehaviour {
                    activo,
                    script,
                    campos,
                } => obj.scripts.push(Script {
                    nombre: script.clone(),
                    activo: *activo,
                    campos: campos.clone(),
                }),
                Componente::Camera {
                    fondo,
                    ortografica,
                    tamano_ortografico,
                    campo_vision,
                    cerca,
                    lejos,
                    ..
                } => {
                    obj.camara = Some(DatosCamara {
                        fondo: color(*fondo),
                        ortografica: *ortografica,
                        tamano_ortografico: *tamano_ortografico,
                        campo_vision: *campo_vision,
                        cerca: *cerca,
                        lejos: *lejos,
                    })
                }
                Componente::ParticleSystem(p) => sistema = Some(p.as_ref().clone()),
                Componente::ParticleSystemRenderer {
                    activo,
                    material,
                    modo,
                    escala_largo,
                    escala_velocidad,
                    tamano_maximo,
                    capa_orden: capa,
                    orden,
                } => {
                    renderer_particulas = Some(crate::particulas::RendererParticulas {
                        activo: *activo,
                        material: material.clone(),
                        modo: *modo,
                        escala_largo: *escala_largo,
                        escala_velocidad: *escala_velocidad,
                        tamano_maximo: *tamano_maximo,
                        capa_orden: capa_orden(capa),
                        orden: *orden,
                    })
                }
                Componente::Otro => {}
            }
        }
        if let Some(datos) = sistema {
            obj.particulas = Some(Particulas::nuevas(datos, renderer_particulas));
        }
        obj
    }

    /// Object.Instantiate de un prefab: copia su jerarquía (la raíz en `posicion`, o donde la
    /// tenía el prefab) y devuelve la raíz. Los ParticleSystem con Play On Awake empiezan aquí.
    pub fn instanciar(
        &mut self,
        proyecto: &Proyecto,
        prefab: &str,
        posicion: Option<Vec3>,
        rotacion: Option<Quat>,
    ) -> Id {
        let datos = &proyecto.prefabs[prefab];
        let ids = self.crear(proyecto, datos);
        let raiz = ids[&datos[0].id];
        let obj = self.objeto_mut(raiz);
        if let Some(p) = posicion {
            obj.local.posicion = p;
        }
        if let Some(r) = rotacion {
            obj.local.rotacion = r;
        }
        raiz
    }

    /// El objeto de la escena cargada con este id del archivo original.
    pub fn original(&self, id: i64) -> Id {
        self.originales[&id]
    }

    pub fn existe(&self, id: Id) -> bool {
        self.objetos.get(id).is_some_and(|o| o.is_some())
    }

    pub fn objeto(&self, id: Id) -> &Objeto {
        self.objetos[id].as_ref().expect("objeto vivo")
    }

    pub fn objeto_mut(&mut self, id: Id) -> &mut Objeto {
        self.objetos[id].as_mut().expect("objeto vivo")
    }

    /// Los objetos vivos, en el orden en que se crearon.
    pub fn ids(&self) -> Vec<Id> {
        (0..self.objetos.len())
            .filter(|&i| self.objetos[i].is_some())
            .collect()
    }

    /// GameObject.activeInHierarchy.
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

    /// transform.lossyScale (sin las rotaciones de los padres, que estos juegos no combinan con
    /// escalas distintas por eje).
    pub fn escala(&self, id: Id) -> Vec3 {
        let o = self.objeto(id);
        match o.padre {
            Some(p) => self.escala(p) * o.local.escala,
            None => o.local.escala,
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

    /// transform.parent = padre: conserva la posición, la rotación y la escala del mundo.
    pub fn fijar_padre(&mut self, id: Id, padre: Option<Id>) {
        let mundo = self.matriz_mundo(id);
        if let Some(viejo) = self.objeto(id).padre {
            self.objeto_mut(viejo).hijos.retain(|&h| h != id);
        }
        let local = match padre {
            Some(p) => self.matriz_mundo(p).inverse() * mundo,
            None => mundo,
        };
        let (escala, rotacion, posicion) = local.to_scale_rotation_translation();
        let obj = self.objeto_mut(id);
        obj.padre = padre;
        obj.local = Transformacion {
            posicion,
            rotacion,
            escala,
        };
        if let Some(p) = padre {
            self.objeto_mut(p).hijos.push(id);
        }
    }

    /// GameObject.FindGameObjectsWithTag: los objetos activos con la etiqueta, en orden. Como en
    /// Unity, los destruidos en este frame siguen apareciendo hasta que se borran.
    pub fn con_etiqueta(&self, etiqueta: &str) -> Vec<Id> {
        self.ids()
            .into_iter()
            .filter(|&i| self.objeto(i).etiqueta == etiqueta && self.activo_en_jerarquia(i))
            .collect()
    }

    /// GameObject.Find.
    pub fn buscar(&self, nombre: &str) -> Option<Id> {
        self.ids()
            .into_iter()
            .find(|&i| self.objeto(i).nombre == nombre && self.activo_en_jerarquia(i))
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

    /// Borra los objetos destruidos durante el frame (con sus cuerpos y colisionadores).
    pub fn limpiar_destruidos(&mut self) {
        for id in self.ids() {
            if !self.objeto(id).destruido {
                continue;
            }
            self.fisica
                .quitar_objeto(id, self.objetos[id].as_mut().expect("vivo"));
            if let Some(p) = self.objeto(id).padre
                && self.existe(p)
            {
                self.objeto_mut(p).hijos.retain(|&h| h != id);
            }
            self.objetos[id] = None;
        }
    }

    /// Los valores que un clip deja fijos en el BoxCollider2D del objeto (m_Size.x, m_Size.y,
    /// m_Offset.x, m_Offset.y), con Write Defaults: las propiedades que el estado actual no anima
    /// vuelven al valor que tenían cuando el Animator las tocó por primera vez. Las propiedades
    /// que el componente no tiene (como m_Center, el nombre de Unity 4) no se aplican.
    pub fn animar_colisionador(&mut self, id: Id, valores: &[ValorAnimado]) {
        let obj = self.objeto_mut(id);
        let Some(i) = obj
            .colisionadores
            .iter()
            .position(|c| matches!(c.forma, Forma2D::Caja(_)))
        else {
            return;
        };
        let Some(animator) = obj.animator.as_mut() else {
            return;
        };
        let col = &mut obj.colisionadores[i];
        let leer = |c: &Colisionador2D, p: &str| match (&c.forma, p) {
            (Forma2D::Caja(t), "m_Size.x") => Some(t.x),
            (Forma2D::Caja(t), "m_Size.y") => Some(t.y),
            (_, "m_Offset.x") => Some(c.centro.x),
            (_, "m_Offset.y") => Some(c.centro.y),
            _ => None,
        };
        let escribir = |c: &mut Colisionador2D, p: &str, v: f32| match (&mut c.forma, p) {
            (Forma2D::Caja(t), "m_Size.x") => t.x = v,
            (Forma2D::Caja(t), "m_Size.y") => t.y = v,
            (_, "m_Offset.x") => c.centro.x = v,
            (_, "m_Offset.y") => c.centro.y = v,
            _ => {}
        };
        for v in valores.iter().filter(|v| v.componente == "BoxCollider2D") {
            let Some(actual) = leer(col, &v.propiedad) else {
                continue;
            };
            animator
                .defectos
                .entry(v.propiedad.clone())
                .or_insert(actual);
        }
        for (p, defecto) in &animator.defectos {
            let valor = valores
                .iter()
                .find(|v| v.componente == "BoxCollider2D" && &v.propiedad == p)
                .map_or(*defecto, |v| v.valor);
            escribir(col, p, valor);
        }
    }

    /// El ángulo de giro en z de una rotación (los ejes de la física 2D).
    pub fn angulo_z(q: Quat) -> f32 {
        let x = q * vec3(1.0, 0.0, 0.0);
        x.y.atan2(x.x)
    }
}
