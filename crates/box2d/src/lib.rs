//! Traducción a Rust de la parte de Box2D 2.3.1 (Erin Catto, licencia zlib) que usa el
//! comportamiento Physics de Construct 2 con el motor asm.js: el `box2d.js` de Alon Zakai
//! (kripken), que trae Box2D 2.3.1 con la soldadura de vértices de `b2PolygonShape::Set`
//! comentada, compilado por Scirra con `b2_maxPolygonVertices` = 16.
//!
//! Incluye círculos y polígonos, la fase amplia (árbol dinámico), los contactos con su solver
//! (arranque en caliente y solver de bloque), las islas con el sueño de los cuerpos, el impacto
//! continuo (TOI) y la articulación de revolución con límites y motor. Para la física 2D de Unity
//! (`crates/unity`) incluye también los sensores, los eventos de contacto por fixture, la consulta
//! por AABB con `b2TestOverlap`, `DestroyFixture`, `ApplyForceToCenter`, `SetMassData` y el
//! `b2PolygonShape::Set` del Box2D 2.3.1 publicado, con la soldadura de vértices
//! (`PolygonShape::set_soldando`). Quedan fuera las aristas, las cadenas, el resto de
//! articulaciones y el rayo.
//!
//! Los nombres siguen a los del original (en `snake_case`) para poder compararlo línea a línea.
//! Los cálculos son en `f32`, como el `float32` de Box2D; el original compilado a asm.js hace las
//! cuentas intermedias en `f64` y redondea al guardar en memoria, así que los resultados no
//! coinciden bit a bit con los del navegador. Con el C++ compilado a código nativo sí:
//! `comparar/comparar.sh` lo comprueba.
//!
//! Es una versión modificada de Box2D, no el original: el aviso de su licencia está en
//! `LICENCIA-ZLIB.txt`.

pub mod broad_phase;
pub mod collision;
pub mod contact_solver;
pub mod distance;
pub mod dynamic_tree;
pub mod joint;
pub mod math;
pub mod settings;
pub mod shapes;
pub mod toi;
pub mod world;

pub use math::{Vec2, vec2};
pub use shapes::{CircleShape, PolygonShape, Shape};
pub use world::{BodyDef, BodyId, BodyType, ContactEvent, FixtureDef, FixtureId, JointId, World};
