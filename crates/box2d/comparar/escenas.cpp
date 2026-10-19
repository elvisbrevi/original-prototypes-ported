// Escenas de comparación: imprime x, y, ángulo de cada cuerpo en cada paso (en hexadecimal).
#include <Box2D/Box2D.h>
#include <cstdio>
#include <cstring>
#include <vector>

static std::vector<b2Body*> cuerpos;

static void imprimir(int paso) {
    printf("%d", paso);
    for (b2Body* b : cuerpos) {
        b2Vec2 p = b->GetPosition();
        printf(" %a %a %a", p.x, p.y, b->GetAngle());
    }
    printf("\n");
}

static b2Body* caja_estatica(b2World& w, float x, float y, float hx, float hy) {
    b2BodyDef bd;
    bd.position.Set(x, y);
    b2Body* b = w.CreateBody(&bd);
    b2PolygonShape s;
    s.SetAsBox(hx, hy);
    b2FixtureDef fd;
    fd.shape = &s;
    fd.friction = 0.5f;
    b->CreateFixture(&fd);
    return b;
}

static b2Body* poligono(b2World& w, float x, float y, float angulo, const b2Vec2* vs, int n, float densidad, float friccion) {
    b2BodyDef bd;
    bd.type = b2_dynamicBody;
    bd.position.Set(x, y);
    bd.angle = angulo;
    bd.angularDamping = 0.01f;
    b2Body* b = w.CreateBody(&bd);
    b2PolygonShape s;
    s.Set(vs, n);
    b2FixtureDef fd;
    fd.shape = &s;
    fd.density = densidad;
    fd.friction = friccion;
    fd.restitution = 0.2f;
    b->CreateFixture(&fd);
    return b;
}

static b2Body* circulo(b2World& w, float x, float y, float r) {
    b2BodyDef bd;
    bd.type = b2_dynamicBody;
    bd.position.Set(x, y);
    bd.angularDamping = 0.01f;
    b2Body* b = w.CreateBody(&bd);
    b2CircleShape s;
    s.m_radius = r;
    b2FixtureDef fd;
    fd.shape = &s;
    fd.density = 1.0f;
    fd.friction = 500.0f;
    fd.restitution = 0.2f;
    b->CreateFixture(&fd);
    return b;
}

int main(int argc, char** argv) {
    const char* escena = argc > 1 ? argv[1] : "caja";
    b2World w(b2Vec2(0.0f, 10.0f));
    if (!strcmp(escena, "caja")) {
        cuerpos.push_back(caja_estatica(w, 0, 0, 20, 0.5f));
        b2Vec2 vs[5] = {b2Vec2(-0.6f, -0.4f), b2Vec2(0.7f, -0.5f), b2Vec2(0.8f, 0.3f), b2Vec2(0.0f, 0.9f), b2Vec2(-0.7f, 0.4f)};
        cuerpos.push_back(poligono(w, 0, -5, 0.3f, vs, 5, 1.0f, 0.5f));
        for (int i = 0; i < 400; ++i) { w.Step(1.0f / 60.0f, 8, 3); imprimir(i); }
    } else if (!strcmp(escena, "pila")) {
        cuerpos.push_back(caja_estatica(w, 0, 0, 20, 0.5f));
        b2Vec2 vs[4] = {b2Vec2(-0.5f, -0.5f), b2Vec2(0.5f, -0.5f), b2Vec2(0.5f, 0.5f), b2Vec2(-0.5f, 0.5f)};
        for (int k = 0; k < 6; ++k) cuerpos.push_back(poligono(w, 0.05f * k, -1.0f - 1.05f * k, 0, vs, 4, 1.0f, 0.6f));
        for (int i = 0; i < 400; ++i) { w.Step(1.0f / 60.0f, 8, 3); imprimir(i); }
    } else if (!strcmp(escena, "bala")) {
        cuerpos.push_back(caja_estatica(w, 0, 0, 20, 0.1f));
        b2Vec2 vs[3] = {b2Vec2(-0.1f, -0.1f), b2Vec2(0.1f, -0.1f), b2Vec2(0.0f, 0.15f)};
        b2Body* b = poligono(w, 0, -20, 0.2f, vs, 3, 1.0f, 0.5f);
        b->SetLinearVelocity(b2Vec2(3.0f, 300.0f));
        cuerpos.push_back(b);
        for (int i = 0; i < 200; ++i) { w.Step(1.0f / 60.0f, 8, 3); imprimir(i); }
    } else if (!strcmp(escena, "moto")) {
        for (int k = 0; k < 9; ++k) cuerpos.push_back(caja_estatica(w, 2.0f * k, 9.0f, 1.0f, 0.1f));
        b2Vec2 chasis[6] = {b2Vec2(-0.8f, -0.1f), b2Vec2(0.0f, -0.4f), b2Vec2(0.8f, -0.2f), b2Vec2(0.9f, 0.1f), b2Vec2(0.1f, 0.3f), b2Vec2(-0.7f, 0.2f)};
        b2Body* moto = poligono(w, 6.0f, 8.0f, 0.09f, chasis, 6, 1.0f, 0.0f);
        b2Body* r1 = circulo(w, 6.6f, 8.3f, 0.27f);
        b2Body* r2 = circulo(w, 5.35f, 8.3f, 0.27f);
        b2Vec2 cab[5] = {b2Vec2(-0.2f, -0.6f), b2Vec2(0.1f, -0.8f), b2Vec2(0.2f, 0.4f), b2Vec2(-0.1f, 0.5f), b2Vec2(-0.25f, 0.0f)};
        b2Body* cabeza = poligono(w, 6.05f, 7.6f, 0.09f, cab, 5, 1.0f, 0.5f);
        cuerpos.push_back(moto); cuerpos.push_back(r1); cuerpos.push_back(r2); cuerpos.push_back(cabeza);
        b2RevoluteJointDef j1; j1.Initialize(moto, r1, b2Vec2(6.6f, 8.3f)); w.CreateJoint(&j1);
        b2RevoluteJointDef j2; j2.Initialize(moto, r2, b2Vec2(5.35f, 8.3f)); w.CreateJoint(&j2);
        b2RevoluteJointDef j3; j3.Initialize(moto, cabeza, b2Vec2(6.0f, 7.9f)); j3.enableLimit = true; j3.lowerAngle = 0; j3.upperAngle = 0; w.CreateJoint(&j3);
        float vel = 0;
        for (int i = 0; i < 600; ++i) {
            if (i > 30 && vel < 2800) vel += 15;
            r1->SetAngularVelocity(vel * b2_pi / 180.0f); r1->SetAwake(true);
            r2->SetAngularVelocity(vel * b2_pi / 180.0f); r2->SetAwake(true);
            if (i % 120 > 100) { moto->SetAngularVelocity(-200.0f * b2_pi / 180.0f); moto->SetAwake(true); }
            if (i % 50 == 0) {
                // Como Wrap: un tramo de suelo salta adelante.
                b2Body* t = cuerpos[(i / 50) % 9];
                t->SetTransform(b2Vec2(t->GetPosition().x + 18.0f, 9.0f), 0);
            }
            w.Step(1.0f / 60.0f, 8, 3);
            imprimir(i);
        }
    }
    return 0;
}
