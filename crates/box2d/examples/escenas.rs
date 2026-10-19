// Escenas de comparación con el Box2D 2.3.1 original (ver tests/README en el README del crate):
// imprime x, y y ángulo de cada cuerpo en cada paso, en hexadecimal como printf("%a").
// Uso: cargo run -p box2d --example escenas -- <caja|pila|bala|moto>

use box2d::settings::PI;
use box2d::shapes::{CircleShape, PolygonShape};
use box2d::world::{BodyDef, BodyId, BodyType, FixtureDef, World};
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

fn main() {
    let escena = std::env::args().nth(1).unwrap_or_else(|| "caja".into());
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
