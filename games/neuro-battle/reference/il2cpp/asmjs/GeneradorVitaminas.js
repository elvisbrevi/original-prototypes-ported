// GeneradorVitaminas (Assembly-UnityScript.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: objetos (+16), tiempoLanzar (+20), tiempoLanzarCheck (+24)

// void GeneradorVitaminas::.ctor()
function Irg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  c[a+20>>2]=1;
  return
}

// void GeneradorVitaminas::Update()
function Jrg(a,b){
  a=a|0;
  b=b|0;
  var d=0,e=0,f=0,g=0,h=0,j=$n,k=0,l=0;
  h=i;
  i=i+64|0;
  g=h+40|0;
  e=h;
  d=h+12|0;
  f=h+24|0;
  j=fa(J6i(/*Time::get_time*/0,0));
  b=a+24|0;
  if(!(j<fa(c[b>>2]|0))){
    l=c[a+16>>2]|0;
    k=y3i(/*Random::Range*/0,0,Dph(/*Array::get_Length*/l,0)|0,0)|0;
    j6i(/*Transform::get_position*/d,V5i(/*Component::get_transform*/a,0)|0,0);
    M0i(/*Quaternion::get_identity*/f,0,0);
    k=c[l+16+(ca(c[(c[l>>2]|0)+96>>2]|0,k)|0)>>2]|0;
    c[e+0>>2]=c[d+0>>2];
    c[e+4>>2]=c[d+4>>2];
    c[e+8>>2]=c[d+8>>2];
    c[g+0>>2]=c[f+0>>2];
    c[g+4>>2]=c[f+4>>2];
    c[g+8>>2]=c[f+8>>2];
    c[g+12>>2]=c[f+12>>2];
    P5i(/*Object::Instantiate*/0,k,e,g,0)|0;
    j=fa(J6i(/*Time::get_time*/0,0));
    c[b>>2]=~~fa(j+fa(c[a+20>>2]|0))
  }
  i=h;
  return
}

// void GeneradorVitaminas::Main()
function Krg(a,b){
  a=a|0;
  b=b|0;
  return
}
