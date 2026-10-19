// Pruebas de humo del motor: caída sobre el suelo, reposo y sueño, rueda que rueda por una
// pendiente y péndulo con una articulación de revolución.

use box2d::shapes::{CircleShape, PolygonShape};
use box2d::world::{BodyDef, BodyType, FixtureDef, World};
use box2d::{Shape, vec2};

fn suelo(world: &mut World) {
    let id = world.create_body(&BodyDef::default());
    let mut caja = PolygonShape::default();
    caja.set_as_box(20.0, 0.5);
    world.create_fixture(
        id,
        &FixtureDef {
            shape: Shape::Polygon(caja),
            friction: 0.5,
            restitution: 0.0,
            density: 0.0,
            is_sensor: false,
        },
    );
}

#[test]
fn caja_que_cae_y_se_duerme() {
    let mut world = World::new(vec2(0.0, 10.0));
    suelo(&mut world);
    let caja_id = world.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(0.0, -5.0),
        ..Default::default()
    });
    let mut caja = PolygonShape::default();
    caja.set(&[
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5),
        vec2(0.5, 0.5),
        vec2(-0.5, 0.5),
    ]);
    world.create_fixture(
        caja_id,
        &FixtureDef {
            shape: Shape::Polygon(caja),
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        },
    );
    for _ in 0..300 {
        world.step(1.0 / 60.0, 8, 3);
    }
    let b = world.body(caja_id);
    // El suelo va de y = -0,5 a 0,5: la caja descansa con su centro en y = -1 (más el radio
    // de los polígonos).
    assert!(
        (b.get_position().y + 1.0).abs() < 0.05,
        "{:?}",
        b.get_position()
    );
    assert!(b.get_position().x.abs() < 0.01);
    assert!(!b.is_awake(), "la caja quieta tendría que dormirse");
    assert!(!world.begin_contacts.is_empty());
}

#[test]
fn rueda_que_rueda() {
    let mut world = World::new(vec2(0.0, 10.0));
    suelo(&mut world);
    let rueda = world.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(0.0, -1.0),
        ..Default::default()
    });
    world.create_fixture(
        rueda,
        &FixtureDef {
            shape: Shape::Circle(CircleShape::new(0.5)),
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        },
    );
    for _ in 0..60 {
        world.body_mut(rueda).set_angular_velocity(5.0);
        world.step(1.0 / 60.0, 8, 3);
    }
    let b = world.body(rueda);
    // Girando en sentido horario (y hacia abajo) avanza hacia la derecha.
    assert!(b.get_position().x > 0.5, "{:?}", b.get_position());
    assert!((b.get_position().y + 1.0).abs() < 0.05);
}

#[test]
fn pendulo() {
    let mut world = World::new(vec2(0.0, 10.0));
    let ancla = world.create_body(&BodyDef::default());
    let bola = world.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: vec2(2.0, 0.0),
        ..Default::default()
    });
    world.create_fixture(
        bola,
        &FixtureDef {
            shape: Shape::Circle(CircleShape::new(0.25)),
            friction: 0.2,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        },
    );
    let def = world.revolute_joint_def(ancla, bola, vec2(0.0, 0.0));
    world.create_joint(&def);
    let mut max_y: f32 = 0.0;
    for _ in 0..240 {
        world.step(1.0 / 60.0, 8, 3);
        let p = world.body(bola).get_position();
        assert!(
            (p.length() - 2.0).abs() < 0.05,
            "la distancia al ancla cambia: {p:?}"
        );
        max_y = max_y.max(p.y);
    }
    assert!(
        max_y > 1.9,
        "el péndulo tendría que bajar hasta abajo del ancla"
    );
}
