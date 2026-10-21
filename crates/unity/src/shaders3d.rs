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

/// Inserta la función de atenuación en un shader de fragmento.
pub fn con_atenuacion(fs: &str) -> String {
    fs.replace("ATENUACION", ATENUACION)
}
