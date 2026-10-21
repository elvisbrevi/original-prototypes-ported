// bala (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: 

// void bala::.ctor()
function brg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void bala::Start()
function crg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void bala::Update()
function drg(a,b){
  a=a|0;
  b=b|0;
  var d=0,e=0;
  b=i;
  i=i+32|0;
  d=b+12|0;
  e=b;
  a=V5i(/*Component::get_transform*/a,0)|0;
  x0i(/*Vector3::get_up*/e,0,0);
  c[d+0>>2]=c[e+0>>2];
  c[d+4>>2]=c[e+4>>2];
  c[d+8>>2]=c[e+8>>2];
  B6i(/*Transform::Translate*/a,d,0);
  i=b;
  return
}

// void bala::OnTriggerEnter2D(Collider2D other)
function erg(b,d,e){
  b=b|0;
  d=d|0;
  e=e|0;
  if(!(a[1287192]|0)){
    c[678948]=FO(-2147483642)/*Component::GetComponent<muerte_enemigo>*/|0;
    a[1287192]=1
  }
  if(!((!(bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2959752/*"enemigo"*/,0)|0)?!(bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2959784/*"lateral"*/,0)|0):0)?!(bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2959816/*"esquina"*/,0)|0):0)){
    d=V5i(/*Component::get_transform*/d,0)|0;
    d=(OBj(d,c[678948]|0)|0)+16|0;
    g[d>>2]=fa(fa(g[d>>2])+fa(-1.0));
    F5i(/*Object::Destroy*/0,W5i(/*Component::get_gameObject*/b,0)|0,0)
  }
  return
}

// void bala::OnCollisionEnter2D(Collision2D coll)
function frg(a,b,c){
  a=a|0;
  b=b|0;
  c=c|0;
  if(!(!(bjh(/*String::Contains*/W_i(/*Object::get_name*/u7i(/*Collision2D::get_gameObject*/b,0)|0,0)|0,2959848/*"Enemy"*/,0)|0)?!(bjh(/*String::Contains*/W_i(/*Object::get_name*/u7i(/*Collision2D::get_gameObject*/b,0)|0,0)|0,2959816/*"esquina"*/,0)|0):0))F5i(/*Object::Destroy*/0,W5i(/*Component::get_gameObject*/a,0)|0,0);
  return
}
