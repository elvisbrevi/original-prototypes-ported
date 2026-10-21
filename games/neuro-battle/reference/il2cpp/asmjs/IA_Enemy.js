// IA_Enemy (Assembly-UnityScript.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: target (+16), waypoint (+20), velocidadMovimiento_x (+24), velocidadMovimiento_y (+28), velocidadHuida (+32), estadoEnemigo (+36), distanciaTarget (+40), distanciaWaypoint (+44), distanciaMuerte (+48), rangoMovimiento (+52), tiempoQuieto (+56), tiempoQuietoCheck (+60)

// void IA_Enemy::.ctor()
function Lrg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  g[a+24>>2]=fa(3.0);
  g[a+28>>2]=fa(1.5);
  g[a+32>>2]=fa(4.5);
  g[a+40>>2]=fa(10.0);
  g[a+44>>2]=fa(4.0);
  g[a+48>>2]=fa(1.10000002);
  g[a+52>>2]=fa(15.0);
  c[a+56>>2]=5;
  return
}

// void IA_Enemy::Start()
function Mrg(a,b){
  a=a|0;
  b=b|0;
  c[a+36>>2]=3;
  return
}

// void IA_Enemy::CrearWaypoint()
function Nrg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0,g=0,h=0,j=0,k=0;
  k=i;
  i=i+32|0;
  j=k+16|0;
  e=k;
  h=k+4|0;
  if(!(a[1289048]|0)){
    c[676794]=EO(66)|0;
    c[677126]=DO(67)/*typeof(System.Type)*/|0;
    c[677128]=DO(66)/*typeof(UnityEngine.MeshRenderer)*/|0;
    a[1289048]=1
  }
  f=$5i(/*GameObject::CreatePrimitive*/0,0,0)|0;
  g=b+20|0;
  c[g>>2]=a6i(/*GameObject::get_transform*/f,0)|0;
  d=c[677126]|0;
  if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
  c[e>>2]=c[676794];
  c[j+0>>2]=c[e+0>>2];
  b=Y5i(/*GameObject::GetComponent*/f,Ubh(/*Type::GetTypeFromHandle*/0,j,0)|0,0)|0;
  e=c[677128]|0;
  do if(!b)d=0;
  else{
    d=ZO(b,e)|0;
    if(d)break;
    qN(j,c[(c[b>>2]|0)+28>>2]|0,e);
    if(!(a[j>>0]&1))d=j+1|0;
    else d=c[j+8>>2]|0;
    r=0;
    d=dc(1026/*invoke aN*/,d|0)|0;
    e=r;
    r=0;
    if(!(e&1)?(r=0,bc(3842/*invoke VM*/,d|0),e=r,r=0,!(e&1)):0){
      Zkk(j);
      d=0;
      break
    }
    h=hg()|0;
    Zkk(j);
    tg(h|0)
  }
  while(0);
  YWi(/*Renderer::set_enabled*/d,0,0);
  g=V5i(/*Component::get_transform*/c[g>>2]|0,0)|0;
  f=y3i(/*Random::Range*/0,-66,19,0)|0;
  e=y3i(/*Random::Range*/0,-32,18,0)|0;
  c[h+0>>2]=0;
  c[h+4>>2]=0;
  c[h+8>>2]=0;
  X$i(/*Vector3::.ctor*/h,fa(f|0),fa(e|0),fa(0.0),0);
  c[j+0>>2]=c[h+0>>2];
  c[j+4>>2]=c[h+4>>2];
  c[j+8>>2]=c[h+8>>2];
  k6i(/*Transform::set_position*/g,j,0);
  i=k;
  return
}

// void IA_Enemy::Seguir(Transform objetivo)
function Org(b,d,e){
  b=b|0;
  d=d|0;
  e=e|0;
  var f=$n,h=0,j=$n,k=0,l=0,m=0,n=0,o=0,p=0,q=0,r=0,s=0,t=0,u=$n;
  t=i;
  i=i+144|0;
  k=t+120|0;
  e=t+108|0;
  h=t+96|0;
  n=t+84|0;
  o=t+72|0;
  l=t+60|0;
  p=t+36|0;
  q=t+24|0;
  r=t+12|0;
  s=t;
  m=t+48|0;
  if(!(a[1289056]|0)){
    c[677112]=DO(5)/*typeof(UnityEngine.Mathf)*/|0;
    a[1289056]=1
  }
  j6i(/*Transform::get_position*/e,V5i(/*Component::get_transform*/d,0)|0,0);
  j=fa(g[e>>2]);
  j6i(/*Transform::get_position*/h,V5i(/*Component::get_transform*/b,0)|0,0);
  f=fa(g[h>>2]);
  e=c[677112]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0)XP(e);
  j=fa(D2i(/*Mathf::Sign*/0,fa(Fxk(fa(j-f))),0));
  j6i(/*Transform::get_position*/n,V5i(/*Component::get_transform*/b,0)|0,0);
  f=fa(g[n>>2]);
  u=fa(g[b+24>>2]);
  u=fa(f+fa(fa(j*u)*fa(d0i(/*Time::get_deltaTime*/0,0))));
  j6i(/*Transform::get_position*/o,V5i(/*Component::get_transform*/b,0)|0,0);
  h=o+4|0;
  e=c[h>>2]|0;
  h=c[h+4>>2]|0;
  n=V5i(/*Component::get_transform*/b,0)|0;
  g[l>>2]=u;
  o=l+4|0;
  c[o>>2]=e;
  c[o+4>>2]=h;
  c[k+0>>2]=c[l+0>>2];
  c[k+4>>2]=c[l+4>>2];
  c[k+8>>2]=c[l+8>>2];
  k6i(/*Transform::set_position*/n,k,0);
  j6i(/*Transform::get_position*/p,V5i(/*Component::get_transform*/d,0)|0,0);
  u=fa(g[p+4>>2]);
  j6i(/*Transform::get_position*/q,V5i(/*Component::get_transform*/b,0)|0,0);
  u=fa(D2i(/*Mathf::Sign*/0,fa(Fxk(fa(u-fa(g[q+4>>2])))),0));
  j6i(/*Transform::get_position*/r,V5i(/*Component::get_transform*/b,0)|0,0);
  j=fa(g[r+4>>2]);
  f=fa(g[b+28>>2]);
  f=fa(j+fa(fa(u*f)*fa(d0i(/*Time::get_deltaTime*/0,0))));
  j6i(/*Transform::get_position*/s,V5i(/*Component::get_transform*/b,0)|0,0);
  u=fa(g[s>>2]);
  j=fa(g[s+8>>2]);
  d=V5i(/*Component::get_transform*/b,0)|0;
  g[m>>2]=u;
  g[m+4>>2]=f;
  g[m+8>>2]=j;
  c[k+0>>2]=c[m+0>>2];
  c[k+4>>2]=c[m+4>>2];
  c[k+8>>2]=c[m+8>>2];
  k6i(/*Transform::set_position*/d,k,0);
  i=t;
  return
}

// void IA_Enemy::Update()
function Prg(a,b){
  a=a|0;
  b=b|0;
  var d=0,e=0,f=$n,h=0,j=0,k=0,l=0,m=0,n=0,o=0,p=0,q=0,r=0,s=0,t=0,u=0,v=0,w=0,x=0,y=$n,z=$n,A=$n;
  w=i;
  i=i+192|0;
  v=w+168|0;
  u=w+156|0;
  q=w+144|0;
  e=w+132|0;
  r=w+120|0;
  s=w+108|0;
  t=w+96|0;
  h=w+84|0;
  j=w+60|0;
  k=w+48|0;
  l=w+36|0;
  m=w+24|0;
  n=w+12|0;
  o=w;
  p=w+72|0;
  c[q+0>>2]=0;
  c[q+4>>2]=0;
  c[q+8>>2]=0;
  j6i(/*Transform::get_position*/e,V5i(/*Component::get_transform*/a,0)|0,0);
  d=a+16|0;
  j6i(/*Transform::get_position*/r,V5i(/*Component::get_transform*/c[d>>2]|0,0)|0,0);
  c[u+0>>2]=c[e+0>>2];
  c[u+4>>2]=c[e+4>>2];
  c[u+8>>2]=c[e+8>>2];
  c[v+0>>2]=c[r+0>>2];
  c[v+4>>2]=c[r+4>>2];
  c[v+8>>2]=c[r+8>>2];
  f=fa(q0i(/*Vector3::Distance*/0,u,v,0));
  r=a+36|0;
  if((c[r>>2]|0)!=3)R5i(/*Object::op_Inequality*/0,c[a+20>>2]|0,0,0)|0;
  e=a+40|0;
  do if(f>fa(g[e>>2])){
    b=c[r>>2]|0;
    if((b|0)==2){
      x=12;
      break
    }
    else if((b|0)==3){
      b=a+20|0;
      if(SXi(/*Object::op_Equality*/0,c[b>>2]|0,0,0)|0)tak(5,a);
      j6i(/*Transform::get_position*/s,V5i(/*Component::get_transform*/a,0)|0,0);
      j6i(/*Transform::get_position*/t,V5i(/*Component::get_transform*/c[b>>2]|0,0)|0,0);
      c[u+0>>2]=c[s+0>>2];
      c[u+4>>2]=c[s+4>>2];
      c[u+8>>2]=c[s+8>>2];
      c[v+0>>2]=c[t+0>>2];
      c[v+4>>2]=c[t+4>>2];
      c[v+8>>2]=c[t+8>>2];
      f=fa(q0i(/*Vector3::Distance*/0,u,v,0));
      if(!(f>fa(g[a+44>>2])))break;
      uak(6,a,c[b>>2]|0);
      break
    }
    else if((b|0)==1){
      j6i(/*Transform::get_position*/h,V5i(/*Component::get_transform*/a,0)|0,0);
      j6i(/*Transform::get_position*/j,V5i(/*Component::get_transform*/c[d>>2]|0,0)|0,0);
      c[u+0>>2]=c[h+0>>2];
      c[u+4>>2]=c[h+4>>2];
      c[u+8>>2]=c[h+8>>2];
      c[v+0>>2]=c[j+0>>2];
      c[v+4>>2]=c[j+4>>2];
      c[v+8>>2]=c[j+8>>2];
      Z$i(/*Vector3::op_Subtraction*/k,0,u,v,0);
      c[q+0>>2]=c[k+0>>2];
      c[q+4>>2]=c[k+4>>2];
      c[q+8>>2]=c[k+8>>2];
      A=fa(_$i(/*Vector3::get_magnitude*/q,0));
      c[v+0>>2]=c[q+0>>2];
      c[v+4>>2]=c[q+4>>2];
      c[v+8>>2]=c[q+8>>2];
      $$i(/*Vector3::op_Division*/l,0,v,A,0);
      A=fa(g[l>>2]);
      y=fa(g[l+4>>2]);
      u=V5i(/*Component::get_transform*/a,0)|0;
      j6i(/*Transform::get_position*/m,V5i(/*Component::get_transform*/a,0)|0,0);
      z=fa(g[m>>2]);
      j6i(/*Transform::get_position*/n,V5i(/*Component::get_transform*/a,0)|0,0);
      f=fa(g[n+4>>2]);
      c[o+0>>2]=0;
      c[o+4>>2]=0;
      c[o+8>>2]=0;
      X$i(/*Vector3::.ctor*/o,fa(A+z),fa(y+f),fa(0.0),0);
      f=fa(d0i(/*Time::get_deltaTime*/0,0));
      c[v+0>>2]=c[o+0>>2];
      c[v+4>>2]=c[o+4>>2];
      c[v+8>>2]=c[o+8>>2];
      a0i(/*Vector3::op_Multiply*/p,0,v,f,0);
      c[v+0>>2]=c[p+0>>2];
      c[v+4>>2]=c[p+4>>2];
      c[v+8>>2]=c[p+8>>2];
      k6i(/*Transform::set_position*/u,v,0);
      break
    }
    else if(!b){
      A=fa(J6i(/*Time::get_time*/0,0));
      b=a+60|0;
      if(A<fa(c[b>>2]|0))break;
      A=fa(J6i(/*Time::get_time*/0,0));
      c[b>>2]=~~fa(A+fa(c[a+56>>2]|0));
      c[r>>2]=3;
      break
    }
    else break
  }
  else{
    c[r>>2]=2;
    x=12
  }
  while(0);
  if((x|0)==12){
    uak(6,a,c[d>>2]|0);
    if(!(f>fa(g[a+48>>2])))H5i(/*Object::DestroyObject*/0,W5i(/*Component::get_gameObject*/a,0)|0,0);
    if(!(f<fa(g[e>>2])))c[r>>2]=0
  }
  i=w;
  return
}

// void IA_Enemy::Main()
function Qrg(a,b){
  a=a|0;
  b=b|0;
  return
}
