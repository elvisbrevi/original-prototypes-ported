// intrucciones (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: aTexture (+16)

// void intrucciones::.ctor()
function trg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void intrucciones::Start()
function urg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void intrucciones::Update()
function vrg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287240]|0)){
    b=DO(2)/*typeof(UnityEngine.Input)*/|0;
    c[677108]=b;
    a[1287240]=1
  }
  else b=c[677108]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0)XP(b);
  if(w5i(/*Input::GetKey*/0,101,0)|0)Y3i(/*Application::LoadLevel*/0,2,0);
  b=c[677108]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0)XP(b);
  if(w5i(/*Input::GetKey*/0,27,0)|0)Y3i(/*Application::LoadLevel*/0,0,0);
  return
}

// void intrucciones::OnGUI()
function wrg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0,g=0,h=0;
  g=i;
  i=i+32|0;
  f=g+16|0;
  e=g;
  if(!(a[1287248]|0)){
    c[677120]=DO(14)/*typeof(UnityEngine.GUI)*/|0;
    a[1287248]=1
  }
  h=_Wi(/*Screen::get_width*/0,0)|0;
  d=$Wi(/*Screen::get_height*/0,0)|0;
  c[e+0>>2]=0;
  c[e+4>>2]=0;
  c[e+8>>2]=0;
  c[e+12>>2]=0;
  ZXi(/*Rect::.ctor*/e,fa(0.0),fa(0.0),fa(h|0),fa(d|0),0);
  d=c[b+16>>2]|0;
  b=c[677120]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0)XP(b);
  c[f+0>>2]=c[e+0>>2];
  c[f+4>>2]=c[e+4>>2];
  c[f+8>>2]=c[e+8>>2];
  c[f+12>>2]=c[e+12>>2];
  PXi(/*GUI::DrawTexture*/0,f,d,0,1,fa(0.0),0);
  i=g;
  return
}
