// animacionUnion (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: anim (+16)

// void animacionUnion::.ctor()
function _qg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void animacionUnion::Start()
function $qg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287176]|0)){
    d=FO(-2147483646)/*Component::GetComponent<Animator>*/|0;
    c[678940]=d;
    a[1287176]=1
  }
  else d=c[678940]|0;
  c[b+16>>2]=OBj(b,d)|0;
  return
}

// void animacionUnion::Update()
function arg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287184]|0)){
    b=DO(10)/*typeof(BaseRotacion)*/|0;
    c[677116]=b;
    a[1287184]=1
  }
  else b=c[677116]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0)XP(b);
  return
}
