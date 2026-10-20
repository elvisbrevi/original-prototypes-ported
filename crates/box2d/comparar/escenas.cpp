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

// Para la escena "corredor": BeginContact y EndContact con los datos de usuario de las fixtures.
struct Oyente : b2ContactListener {
    int paso = 0;
    void BeginContact(b2Contact* c) override {
        printf("B %d %d %d\n", paso, (int)(intptr_t)c->GetFixtureA()->GetUserData(), (int)(intptr_t)c->GetFixtureB()->GetUserData());
    }
    void EndContact(b2Contact* c) override {
        printf("E %d %d %d\n", paso, (int)(intptr_t)c->GetFixtureA()->GetUserData(), (int)(intptr_t)c->GetFixtureB()->GetUserData());
    }
};

struct Consulta : b2QueryCallback {
    b2CircleShape circulo;
    b2Transform xf;
    int tocados = 0;
    bool ReportFixture(b2Fixture* f) override {
        if (b2TestOverlap(&circulo, 0, f->GetShape(), 0, xf, f->GetBody()->GetTransform())) tocados += (int)(intptr_t)f->GetUserData();
        return true;
    }
};

static b2Fixture* caja_fija(b2Body* b, float hx, float hy, b2Vec2 centro, bool sensor, int dato, float densidad) {
    b2PolygonShape s;
    s.SetAsBox(hx, hy, centro, 0.0f);
    b2FixtureDef fd;
    fd.shape = &s;
    fd.isSensor = sensor;
    fd.friction = 0.0f;
    fd.density = densidad;
    fd.userData = (void*)(intptr_t)dato;
    return b->CreateFixture(&fd);
}

// Como Super Happy Jump: un corredor (círculo y caja, rotación fija, masa 1) sobre tramos de
// suelo, con saltos por fuerza, sensores dinámicos y estáticos, un sensor que se teletransporta y
// la consulta del suelo bajo los pies.
static void corredor() {
    b2World w(b2Vec2(0.0f, -9.81f));
    Oyente oyente;
    w.SetContactListener(&oyente);
    std::vector<b2Body*> suelo;
    for (int k = 0; k < 8; ++k) {
        b2BodyDef bd;
        bd.position.Set(-9.0f + 2.0f * k, -2.0f);
        b2Body* b = w.CreateBody(&bd);
        caja_fija(b, 1.0f, 0.15f, b2Vec2(0, 0), false, 1, 0.0f);
        suelo.push_back(b);
        cuerpos.push_back(b);
    }
    b2BodyDef bd;
    bd.type = b2_dynamicBody;
    bd.position.Set(-5.0f, -0.9f);
    bd.fixedRotation = true;
    bd.gravityScale = 1.5f;
    bd.angularDamping = 0.05f;
    b2Body* j = w.CreateBody(&bd);
    b2CircleShape c;
    c.m_radius = 0.19f;
    c.m_p.Set(-0.06f, -0.34f);
    b2FixtureDef fd;
    fd.shape = &c;
    fd.density = 1.0f;
    fd.friction = 0.4f;
    fd.userData = (void*)(intptr_t)10;
    j->CreateFixture(&fd);
    caja_fija(j, 0.19f, 0.415f, b2Vec2(-0.06f, 0.08f), false, 20, 1.0f);
    b2MassData md;
    j->GetMassData(&md);
    md.I *= 1.0f / md.mass;
    md.mass = 1.0f;
    j->SetMassData(&md);
    cuerpos.push_back(j);
    // Un cactus: cuerpo dinámico sin gravedad con un sensor.
    bd = b2BodyDef();
    bd.type = b2_dynamicBody;
    bd.position.Set(4.0f, -1.5f);
    bd.gravityScale = 0.0f;
    b2Body* cactus = w.CreateBody(&bd);
    caja_fija(cactus, 0.4f, 3.0f, b2Vec2(0, 0), true, 100, 1.0f);
    cuerpos.push_back(cactus);
    // Una moneda: sensor estático.
    bd = b2BodyDef();
    bd.position.Set(8.0f, 0.5f);
    b2Body* moneda = w.CreateBody(&bd);
    caja_fija(moneda, 0.375f, 0.4f, b2Vec2(0, 0), true, 1000, 0.0f);
    // El Destroyer: dinámico, sin gravedad, con un sensor alto, movido por SetTransform.
    bd = b2BodyDef();
    bd.type = b2_dynamicBody;
    bd.position.Set(-16.0f, 0.0f);
    bd.gravityScale = 0.0f;
    b2Body* destructor = w.CreateBody(&bd);
    caja_fija(destructor, 0.5f, 10.0f, b2Vec2(0, 0), true, 10000, 1.0f);
    b2Fixture* polvo = nullptr;
    for (int i = 0; i < 300; ++i) {
        oyente.paso = i;
        b2Vec2 v = j->GetLinearVelocity();
        j->SetLinearVelocity(b2Vec2(i < 200 ? 8.0f : 0.0f, v.y));
        if (i == 40 || i == 52 || i == 120) {
            j->SetLinearVelocity(b2Vec2(j->GetLinearVelocity().x, 0.0f));
            j->ApplyForceToCenter(b2Vec2(0.0f, 400.0f), true);
        }
        if (i == 30) {
            b2CircleShape p;
            p.m_radius = 1.268f;
            p.m_p.Set(-1.0f, -0.65f);
            b2FixtureDef f;
            f.shape = &p;
            f.isSensor = true;
            f.density = 1.0f;
            f.userData = (void*)(intptr_t)30;
            polvo = j->CreateFixture(&f);
        }
        if (i == 60 && polvo) { j->DestroyFixture(polvo); polvo = nullptr; }
        b2Vec2 pj = j->GetPosition();
        destructor->SetTransform(b2Vec2(pj.x - 10.8f, 0.0f), 0.0f);
        destructor->SetAwake(true);
        if (i % 25 == 0) {
            b2Body* t = suelo[(i / 25) % 8];
            t->SetTransform(b2Vec2(t->GetPosition().x + 16.0f, -2.0f), 0.0f);
        }
        Consulta q;
        q.circulo.m_radius = 0.2f;
        q.xf.Set(b2Vec2(pj.x, pj.y - 0.6487f), 0.0f);
        b2AABB aabb;
        aabb.lowerBound = q.xf.p - b2Vec2(0.2f, 0.2f);
        aabb.upperBound = q.xf.p + b2Vec2(0.2f, 0.2f);
        w.QueryAABB(&q, aabb);
        printf("Q %d %d\n", i, q.tocados);
        w.Step(0.02f, 8, 3);
        imprimir(i);
    }
}

// Para la escena "rebote": además de imprimir los contactos, recuerda la última pared que
// empezaron a tocar el corredor (fixtures 10 y 20) y la caja (50).
struct OyenteRebote : Oyente {
    int pared_corredor = 0, pared_caja = 0;
    void BeginContact(b2Contact* c) override {
        Oyente::BeginContact(c);
        int a = (int)(intptr_t)c->GetFixtureA()->GetUserData();
        int b = (int)(intptr_t)c->GetFixtureB()->GetUserData();
        for (int k = 0; k < 2; ++k) {
            int yo = k ? b : a, otro = k ? a : b;
            if ((otro == 2 || otro == 3) && (yo == 10 || yo == 20)) pared_corredor = otro;
            if ((otro == 2 || otro == 3) && yo == 50) pared_caja = otro;
        }
    }
};

static b2Fixture* fixture(b2Body* b, const b2Shape* s, int dato, float friccion, float rebote, float densidad, bool sensor) {
    b2FixtureDef fd;
    fd.shape = s;
    fd.isSensor = sensor;
    fd.friction = friccion;
    fd.restitution = rebote;
    fd.density = densidad;
    fd.userData = (void*)(intptr_t)dato;
    return b->CreateFixture(&fd);
}

static void caja_rebote(b2Body* b, float hx, float hy, b2Vec2 centro, int dato, float friccion, float rebote, float densidad) {
    b2PolygonShape s;
    s.SetAsBox(hx, hy, centro, 0.0f);
    fixture(b, &s, dato, friccion, rebote, densidad, false);
}

// Un sensor poligonal hecho con Set, que suelda los vértices cercanos (Box2D 2.3.1 publicado):
// imprime los vértices que quedan.
static void soldado(b2World& w, float x, float y, const b2Vec2* vs, int n, int dato) {
    b2BodyDef bd;
    bd.position.Set(x, y);
    b2Body* b = w.CreateBody(&bd);
    b2PolygonShape s;
    s.Set(vs, n);
    printf("P %d %d", dato, s.m_count);
    for (int i = 0; i < s.m_count; ++i) printf(" %a %a", (double)s.m_vertices[i].x, (double)s.m_vertices[i].y);
    printf(" %a %a\n", (double)s.m_centroid.x, (double)s.m_centroid.y);
    fixture(b, &s, dato, 0.2f, 0.0f, 0.0f, true);
}

static b2Body* dinamico(b2World& w, float x, float y) {
    b2BodyDef bd;
    bd.type = b2_dynamicBody;
    bd.position.Set(x, y);
    bd.fixedRotation = true;
    bd.gravityScale = 3.0f;
    bd.angularDamping = 0.05f;
    return w.CreateBody(&bd);
}

static void masa_uno(b2Body* b) {
    b2MassData md;
    b->GetMassData(&md);
    md.I *= 1.0f / md.mass;
    md.mass = 1.0f;
    b->SetMassData(&md);
}

// Como One Button Prototype (con el Box2D de Unity: b2_maxPolygonVertices = 8 y la soldadura de
// Set): un corredor (círculo y caja, masa 1, gravedad x3) que corre a 10 sobre un suelo con roce
// y da la vuelta al tocar una pared sin roce, saltos por fuerza, una caja que camina y rebota
// (restitución 1) en una repisa con sus propias paredes, sensores poligonales con vértices
// soldados y la consulta del suelo bajo los pies.
static void rebote() {
    b2World w(b2Vec2(0.0f, -9.81f));
    OyenteRebote oyente;
    w.SetContactListener(&oyente);
    b2BodyDef bd;
    bd.position.Set(0.0f, -3.0f);
    caja_rebote(w.CreateBody(&bd), 14.0f, 0.32f, b2Vec2(0, 0), 1, 0.4f, 0.0f, 0.0f);
    bd.position.Set(-11.0f, -1.0f);
    caja_rebote(w.CreateBody(&bd), 0.3f, 2.0f, b2Vec2(0, 0), 2, 0.0f, 0.0f, 0.0f);
    bd.position.Set(11.0f, -1.0f);
    caja_rebote(w.CreateBody(&bd), 0.3f, 2.0f, b2Vec2(0, 0), 3, 0.0f, 0.0f, 0.0f);
    // El carril de la caja: una repisa alta con sus paredes.
    bd.position.Set(0.0f, 4.0f);
    caja_rebote(w.CreateBody(&bd), 6.0f, 0.3f, b2Vec2(0, 0), 1, 0.4f, 0.0f, 0.0f);
    bd.position.Set(-5.5f, 5.3f);
    caja_rebote(w.CreateBody(&bd), 0.3f, 1.0f, b2Vec2(0, 0), 2, 0.0f, 0.0f, 0.0f);
    bd.position.Set(5.5f, 5.3f);
    caja_rebote(w.CreateBody(&bd), 0.3f, 1.0f, b2Vec2(0, 0), 3, 0.0f, 0.0f, 0.0f);
    b2Body* j = dinamico(w, -6.0f, -1.5f);
    b2CircleShape c;
    c.m_radius = 0.2834f;
    c.m_p.Set(0.0f, -0.7972f);
    fixture(j, &c, 10, 0.4f, 0.0f, 1.0f, false);
    caja_rebote(j, 0.2214f, 0.6023f, b2Vec2(0, 0), 20, 0.4f, 0.0f, 1.0f);
    masa_uno(j);
    cuerpos.push_back(j);
    b2Body* d = dinamico(w, 0.0f, 6.2f);
    caja_rebote(d, 1.2112f, 0.4997f, b2Vec2(0.0134f, -0.4034f), 50, 0.4f, 1.0f, 1.0f);
    masa_uno(d);
    cuerpos.push_back(d);
    b2Vec2 v1[8] = {b2Vec2(-0.5f, -0.5f), b2Vec2(-0.49f, -0.48f), b2Vec2(0.5f, -0.5f), b2Vec2(0.52f, 0.1f),
                    b2Vec2(0.5f, 0.5f), b2Vec2(0.0f, 0.9f), b2Vec2(0.01f, 0.91f), b2Vec2(-0.5f, 0.5f)};
    soldado(w, -2.0f, -2.2f, v1, 8, 100);
    b2Vec2 v2[6] = {b2Vec2(0.0f, -0.4f), b2Vec2(0.6f, 0.2f), b2Vec2(0.62f, 0.21f), b2Vec2(0.0f, 0.8f),
                    b2Vec2(-0.6f, 0.2f), b2Vec2(-0.6f, 0.19f)};
    soldado(w, 7.0f, -2.4f, v2, 6, 200);
    float dir = 1.0f, dir_caja = 1.0f;
    for (int i = 0; i < 400; ++i) {
        oyente.paso = i;
        b2Vec2 v = j->GetLinearVelocity();
        j->SetLinearVelocity(b2Vec2(dir * 10.0f, v.y));
        if (i == 25 || i == 140 || i == 260) {
            j->SetLinearVelocity(b2Vec2(j->GetLinearVelocity().x, 0.0f));
            j->ApplyForceToCenter(b2Vec2(0.0f, 700.0f), true);
        }
        v = d->GetLinearVelocity();
        d->SetLinearVelocity(b2Vec2(dir_caja * 2.5f, v.y));
        b2Vec2 pj = j->GetPosition();
        Consulta q;
        q.circulo.m_radius = 0.2f;
        q.xf.Set(b2Vec2(pj.x, pj.y - 1.069f), 0.0f);
        b2AABB aabb;
        aabb.lowerBound = q.xf.p - b2Vec2(0.2f, 0.2f);
        aabb.upperBound = q.xf.p + b2Vec2(0.2f, 0.2f);
        w.QueryAABB(&q, aabb);
        printf("Q %d %d\n", i, q.tocados);
        w.Step(0.02f, 8, 3);
        imprimir(i);
        if (oyente.pared_corredor) { dir = oyente.pared_corredor == 3 ? -1.0f : 1.0f; oyente.pared_corredor = 0; }
        if (oyente.pared_caja) { dir_caja = oyente.pared_caja == 3 ? -1.0f : 1.0f; oyente.pared_caja = 0; }
    }
}

int main(int argc, char** argv) {
    const char* escena = argc > 1 ? argv[1] : "caja";
    if (!strcmp(escena, "corredor")) { corredor(); return 0; }
    if (!strcmp(escena, "rebote")) { rebote(); return 0; }
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
