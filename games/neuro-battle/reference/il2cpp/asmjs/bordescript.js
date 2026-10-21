// bordescript (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: 

// void bordescript::.ctor()
function grg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void bordescript::Start()
function hrg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void bordescript::Update()
function irg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void bordescript::OnTriggerEnter2D(Collider2D other)
function jrg(a,b,c){
  a=a|0;
  b=b|0;
  c=c|0;
  if(!(!(bjh(/*String::Contains*/W_i(/*Object::get_name*/b,0)|0,2959752/*"enemigo"*/,0)|0)?!(bjh(/*String::Contains*/W_i(/*Object::get_name*/b,0)|0,2959872/*"bala"*/,0)|0):0))F5i(/*Object::Destroy*/0,W5i(/*Component::get_gameObject*/b,0)|0,0);
  return
}

// void bordescript::OnCollisionEnter2D(Collision2D coll)
function krg(a,b,c){
  a=a|0;
  b=b|0;
  c=c|0;
  if(!(!(bjh(/*String::Contains*/W_i(/*Object::get_name*/u7i(/*Collision2D::get_gameObject*/b,0)|0,0)|0,2959752/*"enemigo"*/,0)|0)?!(bjh(/*String::Contains*/W_i(/*Object::get_name*/u7i(/*Collision2D::get_gameObject*/b,0)|0,0)|0,2959872/*"bala"*/,0)|0):0))F5i(/*Object::Destroy*/0,u7i(/*Collision2D::get_gameObject*/b,0)|0,0);
  return
}
