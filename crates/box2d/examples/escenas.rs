// Escenas de comparación con el Box2D 2.3.1 original (ver tests/README en el README del crate):
// imprime x, y y ángulo de cada cuerpo en cada paso, en hexadecimal como printf("%a").
// Uso: cargo run -p box2d --example escenas -- <caja|pila|bala|moto|corredor|rebote|neuronas>

use std::f32::consts::FRAC_PI_2;

use box2d::collision::Aabb;
use box2d::math::Transform;
use box2d::settings::PI;
use box2d::shapes::{CircleShape, PolygonShape};
use box2d::world::{BodyDef, BodyId, BodyType, Filter, FixtureDef, FixtureId, World};
use box2d::{Shape, Vec2, vec2};

/// printf("%a") de un float promovido a double.
fn hex(x: f32) -> String {
    let d = x as f64;
    if d == 0.0 {
        return if d.is_sign_negative() {
            "-0x0p+0".into()
        } else {
            "0x0p+0".into()
        };
    }
    let bits = d.to_bits();
    let signo = if bits >> 63 == 1 { "-" } else { "" };
    let exp = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let frac = bits & ((1u64 << 52) - 1);
    let mut digitos = format!("{frac:013x}");
    while digitos.ends_with('0') {
        digitos.pop();
    }
    let punto = if digitos.is_empty() {
        String::new()
    } else {
        format!(".{digitos}")
    };
    let e = if exp >= 0 {
        format!("+{exp}")
    } else {
        format!("{exp}")
    };
    format!("{signo}0x1{punto}p{e}")
}

fn imprimir(world: &World, cuerpos: &[BodyId], paso: usize) {
    let mut linea = format!("{paso}");
    for &b in cuerpos {
        let body = world.body(b);
        let p = body.get_position();
        linea += &format!(" {} {} {}", hex(p.x), hex(p.y), hex(body.get_angle()));
    }
    println!("{linea}");
}

fn caja_estatica(w: &mut World, x: f32, y: f32, hx: f32, hy: f32) -> BodyId {
    let b = w.create_body(&BodyDef {
        position: vec2(x, y),
        ..Default::default()
    });
    let mut s = PolygonShape::default();
    s.set_as_box(hx, hy);
    w.create_fixture(
        b,
        &FixtureDef {
            shape: Shape::Polygon(s),
            friction: 0.5,
            restitution: 0.0,
            density: 0.0,
            is_sensor: false,
        },
    );
    b
}

fn poligono(
    w: &mut World,
    x: f32,
    y: f32,
    angulo: f32,
    vs: &[Vec2],
    densidad: f32,
    friccion: f32,
) -> BodyId {
    let b = w.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(x, y),
        angle: angulo,
        angular_damping: 0.01,
        ..Default::default()
    });
    let mut s = PolygonShape::default();
    s.set(vs);
    w.create_fixture(
        b,
        &FixtureDef {
            shape: Shape::Polygon(s),
            friction: friccion,
            restitution: 0.2,
            density: densidad,
            is_sensor: false,
        },
    );
    b
}

fn circulo(w: &mut World, x: f32, y: f32, r: f32) -> BodyId {
    let b = w.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(x, y),
        angular_damping: 0.01,
        ..Default::default()
    });
    w.create_fixture(
        b,
        &FixtureDef {
            shape: Shape::Circle(CircleShape::new(r)),
            friction: 500.0,
            restitution: 0.2,
            density: 1.0,
            is_sensor: false,
        },
    );
    b
}

#[allow(clippy::too_many_arguments)] // los mismos argumentos que caja_fija en escenas.cpp
fn caja_fija(
    w: &mut World,
    b: BodyId,
    hx: f32,
    hy: f32,
    centro: Vec2,
    sensor: bool,
    dato: usize,
    densidad: f32,
) -> FixtureId {
    let mut s = PolygonShape::default();
    s.set_as_box_oriented(hx, hy, centro, 0.0);
    let f = w.create_fixture(
        b,
        &FixtureDef {
            shape: Shape::Polygon(s),
            friction: 0.0,
            restitution: 0.0,
            density: densidad,
            is_sensor: sensor,
        },
    );
    w.set_fixture_user_data(f, dato);
    f
}

/// Imprime los BeginContact (B) y EndContact (E) pendientes, como el oyente de escenas.cpp.
fn eventos(w: &mut World, paso: usize) {
    for e in w.fixture_events.as_mut().expect("eventos").drain(..) {
        let letra = if e.begin { "B" } else { "E" };
        println!("{letra} {paso} {} {}", e.user_a, e.user_b);
    }
}

/// Como Super Happy Jump: ver corredor() en escenas.cpp.
fn corredor() {
    let mut w = World::new(vec2(0.0, -9.81));
    w.fixture_events = Some(Vec::new());
    let mut cuerpos = Vec::new();
    let mut suelo = Vec::new();
    for k in 0..8 {
        let b = w.create_body(&BodyDef {
            position: vec2(-9.0 + 2.0 * k as f32, -2.0),
            ..Default::default()
        });
        caja_fija(&mut w, b, 1.0, 0.15, Vec2::ZERO, false, 1, 0.0);
        suelo.push(b);
        cuerpos.push(b);
    }
    let j = w.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(-5.0, -0.9),
        fixed_rotation: true,
        gravity_scale: 1.5,
        angular_damping: 0.05,
        ..Default::default()
    });
    let mut c = CircleShape::new(0.19);
    c.p = vec2(-0.06, -0.34);
    let f = w.create_fixture(
        j,
        &FixtureDef {
            shape: Shape::Circle(c),
            friction: 0.4,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        },
    );
    w.set_fixture_user_data(f, 10);
    caja_fija(&mut w, j, 0.19, 0.415, vec2(-0.06, 0.08), false, 20, 1.0);
    let mut md = w.get_mass_data(j);
    md.i *= 1.0 / md.mass;
    md.mass = 1.0;
    w.set_mass_data(j, &md);
    cuerpos.push(j);
    let cactus = w.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(4.0, -1.5),
        gravity_scale: 0.0,
        ..Default::default()
    });
    caja_fija(&mut w, cactus, 0.4, 3.0, Vec2::ZERO, true, 100, 1.0);
    cuerpos.push(cactus);
    let moneda = w.create_body(&BodyDef {
        position: vec2(8.0, 0.5),
        ..Default::default()
    });
    caja_fija(&mut w, moneda, 0.375, 0.4, Vec2::ZERO, true, 1000, 0.0);
    let destructor = w.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(-16.0, 0.0),
        gravity_scale: 0.0,
        ..Default::default()
    });
    caja_fija(&mut w, destructor, 0.5, 10.0, Vec2::ZERO, true, 10000, 1.0);
    let mut polvo = None;
    for i in 0..300 {
        let v = w.body(j).linear_velocity;
        w.body_mut(j)
            .set_linear_velocity(vec2(if i < 200 { 8.0 } else { 0.0 }, v.y));
        if i == 40 || i == 52 || i == 120 {
            let vx = w.body(j).linear_velocity.x;
            w.body_mut(j).set_linear_velocity(vec2(vx, 0.0));
            w.apply_force_to_center(j, vec2(0.0, 400.0), true);
        }
        if i == 30 {
            let mut p = CircleShape::new(1.268);
            p.p = vec2(-1.0, -0.65);
            let f = w.create_fixture(
                j,
                &FixtureDef {
                    shape: Shape::Circle(p),
                    friction: 0.2,
                    restitution: 0.0,
                    density: 1.0,
                    is_sensor: true,
                },
            );
            w.set_fixture_user_data(f, 30);
            polvo = Some(f);
        }
        if i == 60
            && let Some(f) = polvo.take()
        {
            w.destroy_fixture(f);
            eventos(&mut w, i);
        }
        let pj = w.body(j).get_position();
        w.set_transform(destructor, vec2(pj.x - 10.8, 0.0), 0.0);
        w.body_mut(destructor).set_awake(true);
        if i % 25 == 0 {
            let t = suelo[(i / 25) % 8];
            let x = w.body(t).get_position().x;
            w.set_transform(t, vec2(x + 16.0, -2.0), 0.0);
        }
        let circulo = Shape::Circle(CircleShape::new(0.2));
        let xf = Transform::new(vec2(pj.x, pj.y - 0.6487), 0.0);
        let aabb = Aabb {
            lower_bound: xf.p - vec2(0.2, 0.2),
            upper_bound: xf.p + vec2(0.2, 0.2),
        };
        let mut tocadas = Vec::new();
        w.query_aabb(&aabb, |f| {
            tocadas.push(f);
            true
        });
        let tocados: usize = tocadas
            .into_iter()
            .filter(|&f| w.test_overlap_fixture(f, &circulo, &xf))
            .map(|f| w.fixture(f).user_data())
            .sum();
        println!("Q {i} {tocados}");
        w.step(0.02, 8, 3);
        eventos(&mut w, i);
        imprimir(&w, &cuerpos, i);
    }
}

#[allow(clippy::too_many_arguments)]
fn fixture(
    w: &mut World,
    b: BodyId,
    shape: Shape,
    dato: usize,
    friccion: f32,
    rebote: f32,
    densidad: f32,
    sensor: bool,
) -> FixtureId {
    let f = w.create_fixture(
        b,
        &FixtureDef {
            shape,
            friction: friccion,
            restitution: rebote,
            density: densidad,
            is_sensor: sensor,
        },
    );
    w.set_fixture_user_data(f, dato);
    f
}

#[allow(clippy::too_many_arguments)]
fn caja_rebote(
    w: &mut World,
    b: BodyId,
    hx: f32,
    hy: f32,
    centro: Vec2,
    dato: usize,
    friccion: f32,
    rebote: f32,
    densidad: f32,
) {
    let mut s = PolygonShape::default();
    s.set_as_box_oriented(hx, hy, centro, 0.0);
    fixture(
        w,
        b,
        Shape::Polygon(s),
        dato,
        friccion,
        rebote,
        densidad,
        false,
    );
}

/// Ver soldado() en escenas.cpp.
fn soldado(w: &mut World, x: f32, y: f32, vs: &[Vec2], dato: usize) {
    let b = w.create_body(&BodyDef {
        position: vec2(x, y),
        ..Default::default()
    });
    let mut s = PolygonShape::default();
    s.set_soldando(vs);
    let mut linea = format!("P {dato} {}", s.count);
    for v in &s.vertices[..s.count] {
        linea += &format!(" {} {}", hex(v.x), hex(v.y));
    }
    println!("{linea} {} {}", hex(s.centroid.x), hex(s.centroid.y));
    fixture(w, b, Shape::Polygon(s), dato, 0.2, 0.0, 0.0, true);
}

fn dinamico(w: &mut World, x: f32, y: f32) -> BodyId {
    w.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(x, y),
        fixed_rotation: true,
        gravity_scale: 3.0,
        angular_damping: 0.05,
        ..Default::default()
    })
}

fn masa_uno(w: &mut World, b: BodyId) {
    let mut md = w.get_mass_data(b);
    md.i *= 1.0 / md.mass;
    md.mass = 1.0;
    w.set_mass_data(b, &md);
}

/// Como One Button Prototype: ver rebote() en escenas.cpp.
fn rebote() {
    let mut w = World::new(vec2(0.0, -9.81));
    w.fixture_events = Some(Vec::new());
    let mut cuerpos = Vec::new();
    for (x, y, hx, hy, dato, friccion) in [
        (0.0, -3.0, 14.0, 0.32, 1, 0.4),
        (-11.0, -1.0, 0.3, 2.0, 2, 0.0),
        (11.0, -1.0, 0.3, 2.0, 3, 0.0),
        (0.0, 4.0, 6.0, 0.3, 1, 0.4),
        (-5.5, 5.3, 0.3, 1.0, 2, 0.0),
        (5.5, 5.3, 0.3, 1.0, 3, 0.0),
    ] {
        let b = w.create_body(&BodyDef {
            position: vec2(x, y),
            ..Default::default()
        });
        caja_rebote(&mut w, b, hx, hy, Vec2::ZERO, dato, friccion, 0.0, 0.0);
    }
    let j = dinamico(&mut w, -6.0, -1.5);
    let mut c = CircleShape::new(0.2834);
    c.p = vec2(0.0, -0.7972);
    fixture(&mut w, j, Shape::Circle(c), 10, 0.4, 0.0, 1.0, false);
    caja_rebote(&mut w, j, 0.2214, 0.6023, Vec2::ZERO, 20, 0.4, 0.0, 1.0);
    masa_uno(&mut w, j);
    cuerpos.push(j);
    let d = dinamico(&mut w, 0.0, 6.2);
    caja_rebote(
        &mut w,
        d,
        1.2112,
        0.4997,
        vec2(0.0134, -0.4034),
        50,
        0.4,
        1.0,
        1.0,
    );
    masa_uno(&mut w, d);
    cuerpos.push(d);
    let v1 = [
        vec2(-0.5, -0.5),
        vec2(-0.49, -0.48),
        vec2(0.5, -0.5),
        vec2(0.52, 0.1),
        vec2(0.5, 0.5),
        vec2(0.0, 0.9),
        vec2(0.01, 0.91),
        vec2(-0.5, 0.5),
    ];
    soldado(&mut w, -2.0, -2.2, &v1, 100);
    let v2 = [
        vec2(0.0, -0.4),
        vec2(0.6, 0.2),
        vec2(0.62, 0.21),
        vec2(0.0, 0.8),
        vec2(-0.6, 0.2),
        vec2(-0.6, 0.19),
    ];
    soldado(&mut w, 7.0, -2.4, &v2, 200);
    let (mut dir, mut dir_caja) = (1.0f32, 1.0f32);
    for i in 0..400 {
        let v = w.body(j).linear_velocity;
        w.body_mut(j).set_linear_velocity(vec2(dir * 10.0, v.y));
        if i == 25 || i == 140 || i == 260 {
            let vx = w.body(j).linear_velocity.x;
            w.body_mut(j).set_linear_velocity(vec2(vx, 0.0));
            w.apply_force_to_center(j, vec2(0.0, 700.0), true);
        }
        let v = w.body(d).linear_velocity;
        w.body_mut(d).set_linear_velocity(vec2(dir_caja * 2.5, v.y));
        let pj = w.body(j).get_position();
        let circulo = Shape::Circle(CircleShape::new(0.2));
        let xf = Transform::new(vec2(pj.x, pj.y - 1.069), 0.0);
        let aabb = Aabb {
            lower_bound: xf.p - vec2(0.2, 0.2),
            upper_bound: xf.p + vec2(0.2, 0.2),
        };
        let mut tocadas = Vec::new();
        w.query_aabb(&aabb, |f| {
            tocadas.push(f);
            true
        });
        let tocados: usize = tocadas
            .into_iter()
            .filter(|&f| w.test_overlap_fixture(f, &circulo, &xf))
            .map(|f| w.fixture(f).user_data())
            .sum();
        println!("Q {i} {tocados}");
        w.step(0.02, 8, 3);
        let mut paredes = (0, 0);
        for e in w.fixture_events.as_mut().expect("eventos").drain(..) {
            let letra = if e.begin { "B" } else { "E" };
            println!("{letra} {i} {} {}", e.user_a, e.user_b);
            if e.begin {
                for (yo, otro) in [(e.user_a, e.user_b), (e.user_b, e.user_a)] {
                    if (otro == 2 || otro == 3) && (yo == 10 || yo == 20) {
                        paredes.0 = otro;
                    }
                    if (otro == 2 || otro == 3) && yo == 50 {
                        paredes.1 = otro;
                    }
                }
            }
        }
        imprimir(&w, &cuerpos, i);
        if paredes.0 != 0 {
            dir = if paredes.0 == 3 { -1.0 } else { 1.0 };
        }
        if paredes.1 != 0 {
            dir_caja = if paredes.1 == 3 { -1.0 } else { 1.0 };
        }
    }
}

fn cuerpo(w: &mut World, tipo: BodyType, x: f32, y: f32, angulo: f32) -> BodyId {
    w.create_body(&BodyDef {
        body_type: tipo,
        position: vec2(x, y),
        angle: angulo,
        gravity_scale: 0.0,
        angular_damping: 0.05,
        ..Default::default()
    })
}

#[allow(clippy::too_many_arguments)]
fn caja_neurona(
    w: &mut World,
    b: BodyId,
    hx: f32,
    hy: f32,
    centro: Vec2,
    angulo: f32,
    dato: usize,
    sensor: bool,
) -> FixtureId {
    let mut s = PolygonShape::default();
    s.set_as_box_oriented(hx, hy, centro, angulo);
    fixture(w, b, Shape::Polygon(s), dato, 0.4, 0.0, 1.0, sensor)
}

/// Ver capa() en escenas.cpp.
fn capa(w: &mut World, f: FixtureId, capa: u32) {
    let mascara = match capa {
        0 => 0xFEC9,
        8 => 0xFEC8,
        _ => 0xFDC9,
    };
    w.set_fixture_filter(
        f,
        Filter {
            category_bits: 1 << capa,
            mask_bits: mascara,
            group_index: 0,
        },
    );
}

/// Ver pose_pata() en escenas.cpp.
fn pose_pata(k: usize, x: f32, y: f32, angulo: f32) -> (Vec2, f32) {
    const PATA_X: [f32; 4] = [-1.0, 1.0, 0.0, 0.0];
    const PATA_Y: [f32; 4] = [0.0, 0.0, -1.0, 1.0];
    const PATA_A: [f32; 4] = [FRAC_PI_2, -FRAC_PI_2, PI, 0.0];
    let (c, s) = (angulo.cos(), angulo.sin());
    (
        vec2(
            x + c * PATA_X[k] - s * PATA_Y[k],
            y + s * PATA_X[k] + c * PATA_Y[k],
        ),
        angulo + PATA_A[k],
    )
}

/// Como Neuro Battle: ver neuronas() en escenas.cpp.
fn neuronas() {
    let mut w = World::new(vec2(0.0, -9.81));
    w.fixture_events = Some(Vec::new());
    w.sensores_cinematicos = true;
    let mut cuerpos = Vec::new();
    let pared = cuerpo(&mut w, BodyType::Static, 9.0, 0.0, 0.0);
    let f = caja_neurona(&mut w, pared, 0.96, 3.0, Vec2::ZERO, 0.0, 1, false);
    capa(&mut w, f, 8);
    let base = cuerpo(&mut w, BodyType::Dynamic, 0.0, 0.0, 0.0);
    let f = caja_neurona(&mut w, base, 0.5, 0.5, Vec2::ZERO, 0.0, 10, false);
    capa(&mut w, f, 9);
    masa_uno(&mut w, base);
    cuerpos.push(base);
    let mut patas = Vec::new();
    let mut libres = Vec::new();
    for k in 0..4 {
        let (p, a) = pose_pata(k, 0.0, 0.0, 0.0);
        let b = cuerpo(&mut w, BodyType::Kinematic, p.x, p.y, a);
        let f = caja_neurona(&mut w, b, 0.085, 0.47, Vec2::ZERO, 0.0, 11 + k, true);
        capa(&mut w, f, 0);
        patas.push(b);
        let (p, a) = pose_pata(k, 4.2, 0.6, 0.0);
        let b = cuerpo(&mut w, BodyType::Kinematic, p.x, p.y, a);
        let f = caja_neurona(&mut w, b, 0.075, 0.47, Vec2::ZERO, 0.0, 21 + k, true);
        capa(&mut w, f, 0);
        libres.push(b);
    }
    cuerpos.push(patas[0]);
    cuerpos.push(libres[0]);
    let pieza = cuerpo(&mut w, BodyType::Static, 4.2, 0.6, 0.0);
    let raiz = caja_neurona(&mut w, pieza, 0.5, 0.5, Vec2::ZERO, 0.0, 20, true);
    capa(&mut w, raiz, 9);
    let enemigo = cuerpo(&mut w, BodyType::Static, -3.0, 4.0, 0.0);
    let f_enemigo = caja_neurona(&mut w, enemigo, 0.73, 0.74, Vec2::ZERO, 0.0, 30, true);
    capa(&mut w, f_enemigo, 0);
    cuerpos.push(enemigo);
    let bala = cuerpo(&mut w, BodyType::Dynamic, 2.0, 1.0, 0.0);
    let f = caja_neurona(&mut w, bala, 0.425, 0.4, Vec2::ZERO, 0.0, 40, true);
    capa(&mut w, f, 0);
    cuerpos.push(bala);
    let mut centro = 0.0f32;
    let mut unida = false;
    let mut rel_libre = [Vec2::ZERO; 4];
    let mut rel_angulo = [0.0f32; 4];
    for i in 0..300 {
        let vx = if i < 150 { 6.0f32 } else { -6.0 };
        let vy = if i < 60 {
            0.0f32
        } else if i < 200 {
            1.5
        } else {
            -3.0
        };
        let ab = w.body(base).get_angle();
        let mut pb = w.body(base).get_position();
        let (dx, dy) = (vx * 0.02, vy * 0.02);
        pb = vec2(
            pb.x + ab.cos() * dx - ab.sin() * dy,
            pb.y + ab.sin() * dx + ab.cos() * dy,
        );
        w.set_transform(base, pb, ab);
        w.body_mut(base).set_awake(true);
        centro += if i < 100 { 0.0 } else { 1.745_329_3 * 0.02 };
        for k in 0..4 {
            let (p, a) = pose_pata(k, pb.x, pb.y, ab + centro);
            w.set_transform(patas[k], p, a);
            w.body_mut(patas[k]).set_awake(true);
            if unida {
                let (c, s) = ((ab + centro).cos(), (ab + centro).sin());
                let r = rel_libre[k];
                let p = vec2(pb.x + c * r.x - s * r.y, pb.y + s * r.x + c * r.y);
                w.set_transform(libres[k], p, ab + centro + rel_angulo[k]);
                w.body_mut(libres[k]).set_awake(true);
            }
        }
        let pe = w.body(enemigo).get_position();
        let sx = if pb.x - pe.x >= 0.0 { 1.0f32 } else { -1.0 };
        let sy = if pb.y - pe.y >= 0.0 { 1.0f32 } else { -1.0 };
        w.set_transform(
            enemigo,
            vec2(pe.x + sx * 3.0 * 0.02, pe.y + sy * 1.5 * 0.02),
            0.0,
        );
        let pa = w.body(bala).get_position();
        w.set_transform(bala, vec2(pa.x + 0.3, pa.y), 0.0);
        if i == 160 {
            capa(&mut w, f_enemigo, 9);
        }
        w.step(0.02, 8, 3);
        let mut union = false;
        for e in w.fixture_events.as_mut().expect("eventos").drain(..) {
            let letra = if e.begin { "B" } else { "E" };
            println!("{letra} {i} {} {}", e.user_a, e.user_b);
            let (a, b) = (e.user_a, e.user_b);
            if e.begin
                && (((11..=14).contains(&a) && (21..=24).contains(&b))
                    || ((11..=14).contains(&b) && (21..=24).contains(&a)))
            {
                union = true;
            }
        }
        imprimir(&w, &cuerpos, i);
        if union && !unida {
            unida = true;
            let ac = w.body(base).get_angle() + centro;
            let pb2 = w.body(base).get_position();
            let pp = w.body(pieza).get_position();
            let ang = w.body(base).get_angle();
            let (c, s) = (ang.cos(), ang.sin());
            let d = vec2(pp.x - pb2.x, pp.y - pb2.y);
            w.destroy_fixture(raiz);
            let f = caja_neurona(
                &mut w,
                base,
                0.5,
                0.5,
                vec2(c * d.x + s * d.y, -s * d.x + c * d.y),
                -ang,
                25,
                true,
            );
            capa(&mut w, f, 9);
            masa_uno(&mut w, base);
            let (cc, sc) = (ac.cos(), ac.sin());
            for k in 0..4 {
                let q = w.body(libres[k]).get_position();
                let e = vec2(q.x - pb2.x, q.y - pb2.y);
                rel_libre[k] = vec2(cc * e.x + sc * e.y, -sc * e.x + cc * e.y);
                rel_angulo[k] = w.body(libres[k]).get_angle() - ac;
            }
        }
    }
}

fn main() {
    let escena = std::env::args().nth(1).unwrap_or_else(|| "caja".into());
    if escena == "corredor" {
        corredor();
        return;
    }
    if escena == "rebote" {
        rebote();
        return;
    }
    if escena == "neuronas" {
        neuronas();
        return;
    }
    let mut w = World::new(vec2(0.0, 10.0));
    let mut cuerpos = Vec::new();
    match escena.as_str() {
        "caja" => {
            cuerpos.push(caja_estatica(&mut w, 0.0, 0.0, 20.0, 0.5));
            let vs = [
                vec2(-0.6, -0.4),
                vec2(0.7, -0.5),
                vec2(0.8, 0.3),
                vec2(0.0, 0.9),
                vec2(-0.7, 0.4),
            ];
            cuerpos.push(poligono(&mut w, 0.0, -5.0, 0.3, &vs, 1.0, 0.5));
            for i in 0..400 {
                w.step(1.0 / 60.0, 8, 3);
                imprimir(&w, &cuerpos, i);
            }
        }
        "pila" => {
            cuerpos.push(caja_estatica(&mut w, 0.0, 0.0, 20.0, 0.5));
            let vs = [
                vec2(-0.5, -0.5),
                vec2(0.5, -0.5),
                vec2(0.5, 0.5),
                vec2(-0.5, 0.5),
            ];
            for k in 0..6 {
                let kf = k as f32;
                cuerpos.push(poligono(
                    &mut w,
                    0.05 * kf,
                    -1.0 - 1.05 * kf,
                    0.0,
                    &vs,
                    1.0,
                    0.6,
                ));
            }
            for i in 0..400 {
                w.step(1.0 / 60.0, 8, 3);
                imprimir(&w, &cuerpos, i);
            }
        }
        "bala" => {
            cuerpos.push(caja_estatica(&mut w, 0.0, 0.0, 20.0, 0.1));
            let vs = [vec2(-0.1, -0.1), vec2(0.1, -0.1), vec2(0.0, 0.15)];
            let b = poligono(&mut w, 0.0, -20.0, 0.2, &vs, 1.0, 0.5);
            w.body_mut(b).set_linear_velocity(vec2(3.0, 300.0));
            cuerpos.push(b);
            for i in 0..200 {
                w.step(1.0 / 60.0, 8, 3);
                imprimir(&w, &cuerpos, i);
            }
        }
        "moto" => {
            for k in 0..9 {
                cuerpos.push(caja_estatica(&mut w, 2.0 * k as f32, 9.0, 1.0, 0.1));
            }
            let chasis = [
                vec2(-0.8, -0.1),
                vec2(0.0, -0.4),
                vec2(0.8, -0.2),
                vec2(0.9, 0.1),
                vec2(0.1, 0.3),
                vec2(-0.7, 0.2),
            ];
            let moto = poligono(&mut w, 6.0, 8.0, 0.09, &chasis, 1.0, 0.0);
            let r1 = circulo(&mut w, 6.6, 8.3, 0.27);
            let r2 = circulo(&mut w, 5.35, 8.3, 0.27);
            let cab = [
                vec2(-0.2, -0.6),
                vec2(0.1, -0.8),
                vec2(0.2, 0.4),
                vec2(-0.1, 0.5),
                vec2(-0.25, 0.0),
            ];
            let cabeza = poligono(&mut w, 6.05, 7.6, 0.09, &cab, 1.0, 0.5);
            cuerpos.extend([moto, r1, r2, cabeza]);
            let j1 = w.revolute_joint_def(moto, r1, vec2(6.6, 8.3));
            w.create_joint(&j1);
            let j2 = w.revolute_joint_def(moto, r2, vec2(5.35, 8.3));
            w.create_joint(&j2);
            let mut j3 = w.revolute_joint_def(moto, cabeza, vec2(6.0, 7.9));
            j3.enable_limit = true;
            j3.lower_angle = 0.0;
            j3.upper_angle = 0.0;
            w.create_joint(&j3);
            let mut vel: f32 = 0.0;
            for i in 0..600 {
                if i > 30 && vel < 2800.0 {
                    vel += 15.0;
                }
                for r in [r1, r2] {
                    w.body_mut(r).set_angular_velocity(vel * PI / 180.0);
                    w.body_mut(r).set_awake(true);
                }
                if i % 120 > 100 {
                    w.body_mut(moto).set_angular_velocity(-200.0 * PI / 180.0);
                    w.body_mut(moto).set_awake(true);
                }
                if i % 50 == 0 {
                    let t = cuerpos[(i / 50) % 9];
                    let x = w.body(t).get_position().x;
                    w.set_transform(t, vec2(x + 18.0, 9.0), 0.0);
                }
                w.step(1.0 / 60.0, 8, 3);
                imprimir(&w, &cuerpos, i);
            }
        }
        _ => panic!("escena desconocida"),
    }
}
