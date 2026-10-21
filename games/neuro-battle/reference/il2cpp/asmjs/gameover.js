// gameover (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: estilo (+16), aTexture (+20)

// void gameover::.ctor()
function prg(a,b){
  a=a|0;
  b=b|0;
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void gameover::Start()
function qrg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void gameover::Update()
function rrg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287224]|0)){
    b=DO(2)/*typeof(UnityEngine.Input)*/|0;
    c[677108]=b;
    a[1287224]=1
  }
  else b=c[677108]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0)XP(b);
  if(w5i(/*Input::GetKey*/0,101,0)|0)Y3i(/*Application::LoadLevel*/0,2,0);
  b=c[677108]|0;
  if((a[b+130>>0]|0)!=0?(c[b+152>>2]|0)==0:0)XP(b);
  if(w5i(/*Input::GetKey*/0,27,0)|0)Y3i(/*Application::LoadLevel*/0,0,0);
  return
}

// void gameover::OnGUI()
function srg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0,h=0,j=0,k=0,l=0;
  l=i;
  i=i+64|0;
  k=l+40|0;
  h=l;
  f=l+8|0;
  j=l+24|0;
  if(!(a[1287232]|0)){
    c[677120]=DO(14)/*typeof(UnityEngine.GUI)*/|0;
    c[677122]=DO(15)/*typeof(string)*/|0;
    a[1287232]=1
  }
  g[h>>2]=fa(0.0);
  d=_Wi(/*Screen::get_width*/0,0)|0;
  e=$Wi(/*Screen::get_height*/0,0)|0;
  c[f+0>>2]=0;
  c[f+4>>2]=0;
  c[f+8>>2]=0;
  c[f+12>>2]=0;
  ZXi(/*Rect::.ctor*/f,fa(0.0),fa(0.0),fa(d|0),fa(e|0),0);
  e=c[b+20>>2]|0;
  d=c[677120]|0;
  if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
  c[k+0>>2]=c[f+0>>2];
  c[k+4>>2]=c[f+4>>2];
  c[k+8>>2]=c[f+8>>2];
  c[k+12>>2]=c[f+12>>2];
  PXi(/*GUI::DrawTexture*/0,k,e,0,1,fa(0.0),0);
  d=_Wi(/*Screen::get_width*/0,0)|0;
  c[j+0>>2]=0;
  c[j+4>>2]=0;
  c[j+8>>2]=0;
  c[j+12>>2]=0;
  ZXi(/*Rect::.ctor*/j,fa(0.0),fa(20.0),fa(d|0),fa(100.0),0);
  g[h>>2]=fa(S6i(/*PlayerPrefs::GetFloat*/0,2959952/*"tiempo"*/,0));
  d=Fkh(/*Single::ToString*/h,2959984/*"0"*/,0)|0;
  e=c[677122]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0)XP(e);
  h=Bjh(/*String::Concat*/0,2959896/*"TU PUNTAJE FUE DE "*/,d,296e4/*" SEGUNDOS"*/,0)|0;
  f=c[b+16>>2]|0;
  c[k+0>>2]=c[j+0>>2];
  c[k+4>>2]=c[j+4>>2];
  c[k+8>>2]=c[j+8>>2];
  c[k+12>>2]=c[j+12>>2];
  KXi(/*GUI::Label*/0,k,h,f,0);
  i=l;
  return
}
