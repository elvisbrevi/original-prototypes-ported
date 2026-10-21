// Los shaders de Unity 3.4 que usan los juegos 3D, reescritos en GLSL 100 a partir de su código
// compilado (ARB y ps_2_0) que viene en el build. Unity 3 ilumina «el doble»: las luces suman
// color * intensidad * 2 (los shaders de superficie lo multiplican y los de función fija usan
// el combinador «double»).
//
// - Diffuse y Bumped Specular (de superficie, camino Forward): la pasada ForwardBase pone el
//   ambiente (los armónicos esféricos, que sin luces de vértice son el color ambiente) y la
//   ForwardAdd suma la luz puntual por píxel con la atenuación de Unity, 1 / (1 + 25 d²) con d la
//   distancia sobre el rango y un fundido a 0 entre 0,8 y 1 del rango (la textura
//   _LightTexture0). Como los dos términos son positivos, se hacen en una sola pasada saturando
//   cada uno, igual que con la mezcla One One sobre un búfer de 8 bits.
// - Nature/Tree Soft Occlusion Bark y Leaves: iluminación por vértice (la luz puntual como luz de
//   vértice, con atenuación 1 / (1 + d² 25 / rango²)), la oclusión guardada en las tangentes y el
//   combinador «double».
// - Transparent/Cutout/Soft Edge Unlit: la parte opaca con alpha test y, con «Soft Vegetation»,
//   una segunda pasada mezclada con lo que el alpha test descartó.
// - My Shaders/AlphaSelfIllum (las nubes del juego): función fija con emisión blanca, mezclada.
// - FX/Diamond: dos pasadas con cubemaps por reflexión (las caras de atrás y luego las de adelante
//   sumadas).
// - RenderFX/Skybox: textura + tinte - 0,5.
// - Particles/Alpha Blended: 2 * color del vértice * _TintColor * textura.

pub const ATENUACION: &str = r#"
float atenuacion(float d2) {
    if (d2 >= 1.0) { return 0.0; }
    float a = 1.0 / (1.0 + 25.0 * d2);
    if (d2 >= 0.64) { a *= 1.0 - (d2 - 0.64) / 0.36; }
    return a;
}
"#;

pub const VS_ILUMINADO: &str = r#"#version 100
attribute vec3 posicion;
attribute vec3 normal;
attribute vec2 uv;
uniform mat4 mvp;
uniform mat4 modelo;
uniform mat4 normales;
uniform vec4 st;
varying vec2 v_uv;
varying vec3 v_normal;
varying vec3 v_mundo;
void main() {
    gl_Position = mvp * vec4(posicion, 1.0);
    v_uv = uv * st.xy + st.zw;
    v_normal = (normales * vec4(normal, 0.0)).xyz;
    v_mundo = (modelo * vec4(posicion, 1.0)).xyz;
}
"#;

/// Diffuse y Bumped Specular (con la normal de la malla: sin mapa de relieve, Unity usa el
/// «bump» plano). `especular.w` es el exponente (_Shininess * 128); 0 si no hay brillo.
pub const FS_ILUMINADO: &str = r#"#version 100
precision highp float;
varying vec2 v_uv;
varying vec3 v_normal;
varying vec3 v_mundo;
uniform sampler2D textura;
uniform vec4 color;
uniform vec4 ambiente;
uniform vec4 luz_posicion;
uniform vec4 luz_color;
uniform vec4 especular;
uniform vec4 camara;
ATENUACION
void main() {
    vec4 t = texture2D(textura, v_uv) * color;
    vec3 n = normalize(v_normal);
    vec3 base = clamp(t.rgb * ambiente.rgb, 0.0, 1.0);
    vec3 d = luz_posicion.xyz - v_mundo;
    float atten = atenuacion(dot(d, d) * luz_posicion.w * luz_posicion.w);
    vec3 l = normalize(d);
    vec3 suma = t.rgb * luz_color.rgb * max(0.0, dot(n, l));
    if (especular.w > 0.0) {
        vec3 h = normalize(l + normalize(camara.xyz - v_mundo));
        float s = pow(max(0.0, dot(n, h)), especular.w) * t.a;
        suma += luz_color.rgb * especular.rgb * s;
    }
    gl_FragColor = vec4(base + clamp(suma * atten * 2.0, 0.0, 1.0), 1.0);
}
"#;

/// Tree Soft Occlusion (corteza si hojas.x = 0, hojas si 1): la luz por vértice.
pub const VS_ARBOL: &str = r#"#version 100
attribute vec3 posicion;
attribute vec3 normal;
attribute vec2 uv;
attribute vec4 tangente;
uniform mat4 mvp;
uniform mat4 modelo;
uniform vec4 color;
uniform vec4 ambiente;
uniform vec4 luz_posicion;
uniform vec4 luz_color;
uniform vec4 oclusion;
uniform vec4 hojas;
varying vec2 v_uv;
varying vec4 v_color;
void main() {
    gl_Position = mvp * vec4(posicion, 1.0);
    v_uv = uv;
    vec3 mundo = (modelo * vec4(posicion, 1.0)).xyz;
    vec3 d = luz_posicion.xyz - mundo;
    vec3 l = normalize(d);
    float atten = 1.0 / (1.0 + dot(d, d) * 25.0 * luz_posicion.w * luz_posicion.w);
    float luz;
    if (hojas.x > 0.5) {
        luz = max(0.0, dot(tangente.xyz, l) * oclusion.z + tangente.w * oclusion.x) + oclusion.y;
    } else {
        // La corteza usa la normal sin transformar, como el original.
        luz = (tangente.w * oclusion.x + oclusion.y) * max(0.0, dot(normal, l));
    }
    v_color = vec4((ambiente.rgb + luz * atten * luz_color.rgb) * color.rgb, color.a);
}
"#;

pub const FS_ARBOL: &str = r#"#version 100
precision mediump float;
varying vec2 v_uv;
varying vec4 v_color;
uniform sampler2D textura;
uniform vec4 corte;
void main() {
    vec4 t = texture2D(textura, v_uv);
    if (corte.x > 0.5 && t.a < corte.y) { discard; }
    gl_FragColor = vec4(clamp(v_color.rgb, 0.0, 1.0) * 2.0 * t.rgb, 1.0);
}
"#;

/// Sin iluminación: textura * color, con el alpha test y la mezcla que pide cada pasada.
/// `recorte.x` es _Cutoff y `recorte.y` el modo: 0 sin recorte, 1 descarta alfa < corte (la
/// pasada opaca), 2 descarta alfa > corte (la pasada suave de Soft Edge Unlit).
pub const VS_SIMPLE: &str = r#"#version 100
attribute vec3 posicion;
attribute vec2 uv;
uniform mat4 mvp;
uniform vec4 st;
varying vec2 v_uv;
void main() {
    gl_Position = mvp * vec4(posicion, 1.0);
    v_uv = uv * st.xy + st.zw;
}
"#;

pub const FS_SIMPLE: &str = r#"#version 100
precision mediump float;
varying vec2 v_uv;
uniform sampler2D textura;
uniform vec4 color;
uniform vec4 recorte;
void main() {
    vec4 t = texture2D(textura, v_uv) * color;
    if (recorte.y > 0.5 && recorte.y < 1.5 && t.a < recorte.x) { discard; }
    if (recorte.y > 1.5 && t.a > recorte.x) { discard; }
    gl_FragColor = t;
}
"#;

/// FX/Diamond: el cubemap de refracción por el color y, en la segunda pasada, más el de reflexión.
pub const VS_DIAMANTE: &str = r#"#version 100
attribute vec3 posicion;
attribute vec3 normal;
uniform mat4 mvp;
uniform mat4 modelo;
uniform mat4 normales;
uniform vec4 camara;
varying vec3 v_reflejo;
void main() {
    gl_Position = mvp * vec4(posicion, 1.0);
    vec3 mundo = (modelo * vec4(posicion, 1.0)).xyz;
    vec3 n = normalize((normales * vec4(normal, 0.0)).xyz);
    v_reflejo = reflect(normalize(mundo - camara.xyz), n);
}
"#;

pub const FS_DIAMANTE: &str = r#"#version 100
precision mediump float;
varying vec3 v_reflejo;
uniform samplerCube refraccion;
uniform samplerCube reflexion;
uniform vec4 color;
uniform vec4 pasada;
void main() {
    vec3 c = textureCube(refraccion, v_reflejo).rgb * color.rgb;
    if (pasada.x > 0.5) { c += textureCube(reflexion, v_reflejo).rgb; }
    gl_FragColor = vec4(c, 1.0);
}
"#;

/// RenderFX/Skybox.
pub const FS_CIELO: &str = r#"#version 100
precision mediump float;
varying vec2 v_uv;
uniform sampler2D textura;
uniform vec4 color;
uniform vec4 recorte;
void main() {
    vec4 t = texture2D(textura, v_uv);
    gl_FragColor = vec4(t.rgb + color.rgb - 0.5, t.a * color.a);
}
"#;

/// Partículas y destellos: posición ya en el mundo (o en la pantalla), color por vértice.
pub const VS_COLOR: &str = r#"#version 100
attribute vec3 posicion;
attribute vec2 uv;
attribute vec4 color;
uniform mat4 mvp;
varying vec2 v_uv;
varying vec4 v_color;
void main() {
    gl_Position = mvp * vec4(posicion, 1.0);
    v_uv = uv;
    v_color = color;
}
"#;

/// `tinte` multiplica (Particles/Alpha Blended usa 2 * _TintColor).
pub const FS_COLOR: &str = r#"#version 100
precision mediump float;
varying vec2 v_uv;
varying vec4 v_color;
uniform sampler2D textura;
uniform vec4 tinte;
void main() {
    gl_FragColor = v_color * tinte * texture2D(textura, v_uv);
}
"#;

/// Hidden/TerrainEngine/Splatmap/Lightmap-FirstPass: las capas mezcladas con el mapa de control
/// (cada capa repetida según su tamaño) e iluminadas como Diffuse.
pub const VS_TERRENO: &str = r#"#version 100
attribute vec3 posicion;
attribute vec3 normal;
attribute vec2 uv;
uniform mat4 mvp;
varying vec2 v_uv;
varying vec3 v_normal;
varying vec3 v_mundo;
void main() {
    gl_Position = mvp * vec4(posicion, 1.0);
    v_uv = uv;
    v_normal = normal;
    v_mundo = posicion;
}
"#;

pub const FS_TERRENO: &str = r#"#version 100
precision highp float;
varying vec2 v_uv;
varying vec3 v_normal;
varying vec3 v_mundo;
uniform sampler2D control;
uniform sampler2D capa0;
uniform sampler2D capa1;
uniform vec4 repetir;
uniform vec4 ambiente;
uniform vec4 luz_posicion;
uniform vec4 luz_color;
ATENUACION
void main() {
    vec4 c = texture2D(control, v_uv);
    vec3 albedo = texture2D(capa0, v_uv * repetir.xy).rgb * c.r
        + texture2D(capa1, v_uv * repetir.zw).rgb * c.g;
    vec3 n = normalize(v_normal);
    vec3 d = luz_posicion.xyz - v_mundo;
    float atten = atenuacion(dot(d, d) * luz_posicion.w * luz_posicion.w);
    vec3 suma = albedo * luz_color.rgb * max(0.0, dot(n, normalize(d)));
    gl_FragColor = vec4(clamp(albedo * ambiente.rgb, 0.0, 1.0) + clamp(suma * atten * 2.0, 0.0, 1.0), 1.0);
}
"#;

/// Hidden/TerrainEngine/Details/BillboardWavingDoublePass: cada mata es un cuadro que mira a la
/// cámara (la tangente lleva su desplazamiento) y cuyo borde de arriba se mueve con las ondas del
/// viento (TerrainWaveGrass de Unity 3, con sus constantes). El color es el del vértice por el
/// tinte de la onda, y se apaga hacia la distancia de los detalles.
pub const VS_PASTO: &str = r#"#version 100
attribute vec3 posicion;
attribute vec3 normal;
attribute vec2 uv;
attribute vec4 color;
attribute vec4 tangente;
uniform mat4 mvp;
uniform vec4 onda;
uniform vec4 camara;
uniform vec4 derecha;
uniform vec4 arriba;
uniform vec4 tinte;
varying vec2 v_uv;
varying vec4 v_color;
varying vec3 v_mundo;
void main() {
    vec3 p = posicion;
    vec3 rel = p - camara.xyz;
    vec2 desplazamiento = dot(rel, rel) > onda.w ? vec2(0.0) : tangente.xy;
    p += desplazamiento.x * derecha.xyz + desplazamiento.y * arriba.xyz;
    vec4 ondas = p.x * vec4(0.012, 0.02, 0.06, 0.024) * onda.y
        + p.z * vec4(0.006, 0.02, 0.02, 0.05) * onda.y
        + vec4(1.2, 2.0, 1.6, 4.8) * onda.x;
    ondas = fract(ondas) * 6.408849 - 3.1415927;
    vec4 c3 = ondas * ondas * ondas;
    vec4 s = ondas - 0.16161616 * c3 + 0.0083333 * c3 * ondas * ondas
        - 0.00019841 * c3 * ondas * ondas * ondas * ondas;
    s = s * s;
    s = s * s;
    float luz = dot(s, vec4(0.47193992, 0.47193992, 0.18877596, 0.094387978));
    vec4 m = s * tangente.y;
    p.x -= dot(m, vec4(0.024, 0.04, -0.12, 0.096)) * onda.z;
    p.z -= dot(m, vec4(0.006, 0.02, -0.02, 0.1)) * onda.z;
    gl_Position = mvp * vec4(p, 1.0);
    v_uv = uv;
    vec3 c = mix(vec3(0.5), tinte.rgb, luz) * color.rgb * 2.0;
    rel = p - camara.xyz;
    v_color = vec4(c, clamp(2.0 * (onda.w - dot(rel, rel)) * camara.w, 0.0, 1.0));
    v_mundo = p;
}
"#;

pub const FS_PASTO: &str = r#"#version 100
precision highp float;
varying vec2 v_uv;
varying vec4 v_color;
varying vec3 v_mundo;
uniform sampler2D textura;
uniform vec4 ambiente;
uniform vec4 luz_posicion;
uniform vec4 luz_color;
uniform vec4 corte;
ATENUACION
void main() {
    vec4 t = texture2D(textura, v_uv) * clamp(v_color, 0.0, 1.0);
    if (t.a < corte.x) { discard; }
    vec3 d = luz_posicion.xyz - v_mundo;
    float atten = atenuacion(dot(d, d) * luz_posicion.w * luz_posicion.w);
    vec3 suma = t.rgb * luz_color.rgb * max(0.0, normalize(d).y);
    gl_FragColor = vec4(clamp(t.rgb * ambiente.rgb, 0.0, 1.0) + clamp(suma * atten * 2.0, 0.0, 1.0), 1.0);
}
"#;

/// Inserta la función de atenuación en un shader de fragmento.
pub fn con_atenuacion(fs: &str) -> String {
    fs.replace("ATENUACION", ATENUACION)
}
