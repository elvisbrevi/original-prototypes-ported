// La descomposición de un PolygonCollider2D en piezas convexas para Box2D, que solo acepta
// polígonos convexos de hasta b2_maxPolygonVertices (8 en el Box2D de Unity). Unity no publica su
// algoritmo; aquí cada camino se triangula cortando orejas y los triángulos vecinos se fusionan
// mientras el resultado siga siendo convexo y no pase de 8 vértices (Hertel-Mehlhorn). Las piezas
// cubren exactamente el polígono, así que la región que ocupa (lo que ven los triggers y las
// consultas) es la misma aunque las piezas no coincidan con las de Unity.

use macroquad::prelude::Vec2;

/// El máximo de vértices de una pieza (b2_maxPolygonVertices del Box2D de Unity).
pub const MAX_VERTICES: usize = 8;

fn cruz(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// El área con signo (positiva si el camino va en sentido antihorario).
pub fn area(p: &[Vec2]) -> f32 {
    let n = p.len();
    (0..n).map(|i| cruz(p[i], p[(i + 1) % n])).sum::<f32>() / 2.0
}

/// Si `p` está dentro del triángulo abc (antihorario), bordes incluidos.
fn dentro(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> bool {
    cruz(b - a, p - a) >= 0.0 && cruz(c - b, p - b) >= 0.0 && cruz(a - c, p - c) >= 0.0
}

/// Triangulación por orejas de un camino simple antihorario, sin puntos repetidos. Devuelve los
/// triángulos como índices del camino.
fn triangular(p: &[Vec2]) -> Vec<[usize; 3]> {
    let mut quedan: Vec<usize> = (0..p.len()).collect();
    let mut triangulos = Vec::new();
    while quedan.len() > 3 {
        let n = quedan.len();
        let oreja = (0..n).find(|&i| {
            let (a, b, c) = (quedan[(i + n - 1) % n], quedan[i], quedan[(i + 1) % n]);
            if cruz(p[b] - p[a], p[c] - p[b]) <= 0.0 {
                return false;
            }
            quedan
                .iter()
                .all(|&j| j == a || j == b || j == c || !dentro(p[a], p[b], p[c], p[j]))
        });
        match oreja {
            Some(i) => {
                triangulos.push([quedan[(i + n - 1) % n], quedan[i], quedan[(i + 1) % n]]);
                quedan.remove(i);
            }
            // Sin orejas: quedan vértices alineados o el camino se cruza. Se quita el vértice
            // menos convexo y se sigue.
            None => {
                let i = (0..n)
                    .min_by(|&x, &y| {
                        let giro = |i: usize| {
                            let (a, b, c) =
                                (quedan[(i + n - 1) % n], quedan[i], quedan[(i + 1) % n]);
                            cruz(p[b] - p[a], p[c] - p[b])
                        };
                        giro(x).total_cmp(&giro(y))
                    })
                    .expect("vértices");
                quedan.remove(i);
            }
        }
    }
    if quedan.len() == 3 && area(&[p[quedan[0]], p[quedan[1]], p[quedan[2]]]) > 0.0 {
        triangulos.push([quedan[0], quedan[1], quedan[2]]);
    }
    triangulos
}

fn convexa(p: &[Vec2], q: &[usize]) -> bool {
    let n = q.len();
    (0..n).all(|i| {
        let (a, b, c) = (p[q[i]], p[q[(i + 1) % n]], p[q[(i + 2) % n]]);
        cruz(b - a, c - b) >= 0.0
    })
}

/// Une dos piezas antihorarias que comparten la arista (u, v), recorrida u -> v en `a` y v -> u
/// en `b`.
fn unir(a: &[usize], b: &[usize], u: usize, v: usize) -> Vec<usize> {
    let ia = a.iter().position(|&x| x == v).expect("v en a");
    let ib = b.iter().position(|&x| x == u).expect("u en b");
    let mut r = Vec::with_capacity(a.len() + b.len() - 2);
    // a desde v hasta u (sin repetir u al final), luego b desde u hasta v, sin los extremos.
    for k in 0..a.len() {
        r.push(a[(ia + k) % a.len()]);
    }
    for k in 1..b.len() - 1 {
        r.push(b[(ib + k) % b.len()]);
    }
    r
}

/// La arista que comparten dos piezas (u -> v en `a`).
fn compartida(a: &[usize], b: &[usize]) -> Option<(usize, usize)> {
    let n = a.len();
    (0..n).map(|i| (a[i], a[(i + 1) % n])).find(|&(u, v)| {
        let m = b.len();
        (0..m).any(|j| b[j] == v && b[(j + 1) % m] == u)
    })
}

/// Las piezas convexas (antihorarias, de 3 a `MAX_VERTICES` vértices) de un camino cerrado en
/// cualquier sentido. Los puntos repetidos seguidos se quitan.
pub fn descomponer(camino: &[Vec2]) -> Vec<Vec<Vec2>> {
    let mut p: Vec<Vec2> = Vec::with_capacity(camino.len());
    for &x in camino {
        if p.last().is_none_or(|&u| u != x) {
            p.push(x);
        }
    }
    while p.len() > 1 && p.first() == p.last() {
        p.pop();
    }
    if p.len() < 3 {
        return Vec::new();
    }
    if area(&p) < 0.0 {
        p.reverse();
    }
    let mut piezas: Vec<Vec<usize>> = triangular(&p).into_iter().map(|t| t.to_vec()).collect();
    // Hertel-Mehlhorn: se fusionan pares vecinos mientras la unión sea convexa y quepa.
    let mut cambio = true;
    while cambio {
        cambio = false;
        'buscar: for i in 0..piezas.len() {
            for j in i + 1..piezas.len() {
                if piezas[i].len() + piezas[j].len() - 2 > MAX_VERTICES {
                    continue;
                }
                let Some((u, v)) = compartida(&piezas[i], &piezas[j]) else {
                    continue;
                };
                let union = unir(&piezas[i], &piezas[j], u, v);
                if convexa(&p, &union) {
                    piezas[i] = union;
                    piezas.remove(j);
                    cambio = true;
                    break 'buscar;
                }
            }
        }
    }
    piezas
        .into_iter()
        .map(|q| q.into_iter().map(|i| p[i]).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec2;

    fn suma_areas(piezas: &[Vec<Vec2>]) -> f32 {
        piezas.iter().map(|q| area(q)).sum()
    }

    #[test]
    fn un_cuadrado_queda_entero() {
        let c = [
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
        ];
        let piezas = descomponer(&c);
        assert_eq!(piezas.len(), 1);
        assert_eq!(piezas[0].len(), 4);
        assert!(area(&piezas[0]) > 0.0);
    }

    #[test]
    fn una_l_se_parte_en_convexas_que_cubren_su_area() {
        let l = [
            vec2(0.0, 0.0),
            vec2(2.0, 0.0),
            vec2(2.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 2.0),
            vec2(0.0, 2.0),
        ];
        let piezas = descomponer(&l);
        assert_eq!(piezas.len(), 2);
        assert!((suma_areas(&piezas) - 3.0).abs() < 1e-5);
        for q in &piezas {
            assert!(q.len() <= MAX_VERTICES);
            let n = q.len();
            for i in 0..n {
                assert!(cruz(q[(i + 1) % n] - q[i], q[(i + 2) % n] - q[(i + 1) % n]) >= 0.0);
            }
        }
    }

    #[test]
    fn una_estrella_de_muchos_vertices() {
        let estrella: Vec<Vec2> = (0..20)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 20.0;
                let r = if i % 2 == 0 { 1.5 } else { 0.6 };
                vec2(r * a.cos(), -r * a.sin())
            })
            .collect();
        let piezas = descomponer(&estrella);
        assert!((suma_areas(&piezas) - area(&estrella).abs()).abs() < 1e-4);
        assert!(piezas.iter().all(|q| (3..=MAX_VERTICES).contains(&q.len())));
    }
}
