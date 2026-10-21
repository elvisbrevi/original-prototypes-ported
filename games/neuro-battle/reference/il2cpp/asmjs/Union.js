// Union (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: estado (+16), shootSound (+20), source (+24)

// void Union::.ctor()
function Vqg(a,b){
  a=a|0;
  b=b|0;
  c[a+16>>2]=2959536/*"libre"*/;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void Union::Awake()
function Wqg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287152]|0)){
    d=FO(-2147483644)/*Component::GetComponent<AudioSource>*/|0;
    c[678944]=d;
    a[1287152]=1
  }
  else d=c[678944]|0;
  c[b+24>>2]=OBj(b,d)|0;
  return
}

// void Union::Start()
function Xqg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void Union::Update()
function Yqg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287160]|0)){
    b=DO(10)/*typeof(BaseRotacion)*/|0;
    c[677116]=b;
    a[1287160]=1
  }
  else b=c[677116]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0){
    XP(b);
    b=c[677116]|0
  }
  a[c[b+64>>2]>>0]=0;
  return
}

// void Union::desconectar()
function Mqg(a,b){
  a=a|0;
  b=b|0;
  x6i(/*Transform::set_parent*/v6i(/*Transform::get_parent*/v6i(/*Transform::get_parent*/V5i(/*Component::get_transform*/a,0)|0,0)|0,0)|0,0,0);
  return
}

// void Union::OnTriggerEnter2D(Collider2D other)
function Zqg(b,d,e){
  b=b|0;
  d=d|0;
  e=e|0;
  if(!(a[1287168]|0)){
    c[677116]=DO(10)/*typeof(BaseRotacion)*/|0;
    c[678946]=FO(-2147483643)/*Component::GetComponent<disparar>*/|0;
    a[1287168]=1
  }
  e=bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2959696/*"pata"*/,0)|0;
  if(e&(bjh(/*String::Contains*/W_i(/*Object::get_name*/v6i(/*Transform::get_parent*/v6i(/*Transform::get_parent*/V5i(/*Component::get_transform*/b,0)|0,0)|0,0)|0,0)|0,2959720/*"ataque"*/,0)|0)){
    e=v6i(/*Transform::get_parent*/v6i(/*Transform::get_parent*/V5i(/*Component::get_transform*/b,0)|0,0)|0,0)|0;
    a[(OBj(e,c[678946]|0)|0)+33>>0]=1
  }
  e=c[677116]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0){
    XP(e);
    e=c[677116]|0
  }
  a[c[e+64>>2]>>0]=1;
  if(bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2959696/*"pata"*/,0)|0?SXi(/*Object::op_Equality*/0,v6i(/*Transform::get_parent*/v6i(/*Transform::get_parent*/v6i(/*Transform::get_parent*/V5i(/*Component::get_transform*/b,0)|0,0)|0,0)|0,0)|0,0,0)|0:0){
    e=v6i(/*Transform::get_parent*/v6i(/*Transform::get_parent*/V5i(/*Component::get_transform*/b,0)|0,0)|0,0)|0;
    z6i(/*Transform::SetParent*/e,V5i(/*Component::get_transform*/v6i(/*Transform::get_parent*/V5i(/*Component::get_transform*/d,0)|0,0)|0,0)|0,0);
    N7i(/*AudioSource::PlayOneShot*/c[b+24>>2]|0,c[b+20>>2]|0,fa(100.0),0)
  }
  e=c[677116]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0){
    XP(e);
    e=c[677116]|0
  }
  a[c[e+64>>2]>>0]=1;
  return
}
