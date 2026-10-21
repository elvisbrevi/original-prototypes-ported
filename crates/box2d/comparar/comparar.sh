#!/bin/sh
# Compara la traducción con el Box2D 2.3.1 original: compila el Box2D del box2d.js de kripken
# (el que trae Construct 2, con b2_maxPolygonVertices = 16 como en la compilación de Scirra) con
# escenas.cpp, y las mismas escenas en Rust (examples/escenas.rs); las trayectorias tienen que
# coincidir bit a bit. La escena "rebote" (One Button Prototype) se compara con una segunda
# compilación como la de Unity: b2_maxPolygonVertices = 8 y la soldadura de vértices de
# b2PolygonShape::Set, que el box2d.js tiene comentada y el Box2D 2.3.1 publicado no. La escena
# "neuronas" (Neuro Battle) usa esa misma compilación, con los contactos entre cuerpos kinematic
# cuando hay un trigger de Unity 5.1.
# Uso: crates/box2d/comparar/comparar.sh [carpeta de trabajo]
set -e
raiz="$(cd "$(dirname "$0")/../../.." && pwd)"
trabajo="${1:-/tmp/box2d-comparar}"
commit=1e6e9da36a3160a9f40cda4cddcb4f83abb4d82c
mkdir -p "$trabajo"
if [ ! -d "$trabajo/box2d.js" ]; then
    git clone -q https://github.com/kripken/box2d.js.git "$trabajo/box2d.js"
fi
git -C "$trabajo/box2d.js" checkout -q "$commit"
rm -rf "$trabajo/Box2D"
cp -r "$trabajo/box2d.js/Box2D_v2.3.1/Box2D" "$trabajo/Box2D"
sed -i 's/#define b2_maxPolygonVertices\t8/#define b2_maxPolygonVertices\t16/' "$trabajo/Box2D/Common/b2Settings.h"
g++ -O2 -std=c++11 -I"$trabajo" "$raiz/crates/box2d/comparar/escenas.cpp" \
    $(find "$trabajo/Box2D" -name '*.cpp') -o "$trabajo/escenas"
# El Box2D de Unity: el de kripken con la soldadura descomentada (y los 8 vértices de siempre).
rm -rf "$trabajo/unity"
mkdir -p "$trabajo/unity"
cp -r "$trabajo/box2d.js/Box2D_v2.3.1/Box2D" "$trabajo/unity/Box2D"
poligono="$trabajo/unity/Box2D/Collision/Shapes/b2PolygonShape.cpp"
sed -i -e 's|^\(\t*\)// if (b2DistanceSquared|\1if (b2DistanceSquared|' -e 's|^\(\t*\)// {$|\1{|' \
    -e 's|^\(\t*\)// \tunique = false;|\1\tunique = false;|' -e 's|^\(\t*\)// \tbreak;|\1\tbreak;|' \
    -e 's|^\(\t*\)// }$|\1}|' "$poligono"
grep -q '^[[:space:]]*if (b2DistanceSquared(v, ps\[j\]) < 0.5f \* b2_linearSlop)' "$poligono"
# Y lo que se observó en Neuro Battle (Unity 5.1): dos cuerpos kinematic se tocan si una de las
# fixtures es un sensor (un trigger), en AddPair y al volver a filtrar en Collide.
contactos="$trabajo/unity/Box2D/Dynamics/b2ContactManager.cpp"
sed -i 's|bodyB->ShouldCollide(bodyA) == false|(bodyB->ShouldCollide(bodyA) == false \&\& !b2SensoresCinematicos(bodyB, bodyA, fixtureA, fixtureB))|' "$contactos"
sed -i '/#include <Box2D\/Dynamics\/Contacts\/b2Contact.h>/r /dev/stdin' "$contactos" <<'FIN'
#include <Box2D/Dynamics/Joints/b2Joint.h>
static bool b2SensoresCinematicos(const b2Body* b, const b2Body* other, const b2Fixture* fa, const b2Fixture* fb)
{
	if (b->GetType() != b2_kinematicBody || other->GetType() != b2_kinematicBody || !(fa->IsSensor() || fb->IsSensor()))
	{
		return false;
	}
	for (const b2JointEdge* jn = b->GetJointList(); jn; jn = jn->next)
	{
		if (jn->other == other && jn->joint->GetCollideConnected() == false) return false;
	}
	return true;
}
FIN
[ "$(grep -c b2SensoresCinematicos "$contactos")" = 3 ]
g++ -O2 -std=c++11 -I"$trabajo/unity" "$raiz/crates/box2d/comparar/escenas.cpp" \
    $(find "$trabajo/unity/Box2D" -name '*.cpp') -o "$trabajo/escenas_unity"
cargo build --release -q -p box2d --example escenas --manifest-path "$raiz/Cargo.toml"
for escena in caja pila bala moto corredor rebote neuronas; do
    if [ "$escena" = rebote ] || [ "$escena" = neuronas ]; then c="$trabajo/escenas_unity"; else c="$trabajo/escenas"; fi
    "$c" "$escena" > "$trabajo/c_$escena.txt"
    "$raiz/target/release/examples/escenas" "$escena" > "$trabajo/rs_$escena.txt"
    if cmp -s "$trabajo/c_$escena.txt" "$trabajo/rs_$escena.txt"; then
        echo "$escena: idéntica ($(wc -l < "$trabajo/rs_$escena.txt") pasos)"
    else
        echo "$escena: DIFIERE"
        exit 1
    fi
done
