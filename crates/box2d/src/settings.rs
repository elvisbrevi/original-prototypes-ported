// b2Settings.h: las constantes del motor (unidades MKS). La única diferencia con Box2D 2.3.1 es
// b2_maxPolygonVertices: el box2d.js (asm.js) que trae Construct 2 está compilado con 16.

pub const MAX_FLOAT: f32 = f32::MAX;
pub const EPSILON: f32 = f32::EPSILON;
pub const PI: f32 = std::f32::consts::PI;

pub const MAX_MANIFOLD_POINTS: usize = 2;
pub const MAX_POLYGON_VERTICES: usize = 16;
pub const AABB_EXTENSION: f32 = 0.1;
pub const AABB_MULTIPLIER: f32 = 2.0;
pub const LINEAR_SLOP: f32 = 0.005;
pub const ANGULAR_SLOP: f32 = 2.0 / 180.0 * PI;
pub const POLYGON_RADIUS: f32 = 2.0 * LINEAR_SLOP;
pub const MAX_SUB_STEPS: i32 = 8;
pub const MAX_TOI_CONTACTS: usize = 32;
pub const VELOCITY_THRESHOLD: f32 = 1.0;
pub const MAX_LINEAR_CORRECTION: f32 = 0.2;
pub const MAX_ANGULAR_CORRECTION: f32 = 8.0 / 180.0 * PI;
pub const MAX_TRANSLATION: f32 = 2.0;
pub const MAX_TRANSLATION_SQUARED: f32 = MAX_TRANSLATION * MAX_TRANSLATION;
pub const MAX_ROTATION: f32 = 0.5 * PI;
pub const MAX_ROTATION_SQUARED: f32 = MAX_ROTATION * MAX_ROTATION;
pub const BAUMGARTE: f32 = 0.2;
pub const TOI_BAUGARTE: f32 = 0.75;
pub const TIME_TO_SLEEP: f32 = 0.5;
pub const LINEAR_SLEEP_TOLERANCE: f32 = 0.01;
pub const ANGULAR_SLEEP_TOLERANCE: f32 = 2.0 / 180.0 * PI;
