// BaseRotacion (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: velocidadRotacion (+16), static choque, upButton (+20)

// void BaseRotacion::.ctor()
function Hqg(b,c){
  b=b|0;
  c=c|0;
  g[b+16>>2]=fa(1.0);
  a[b+20>>0]=1;
  l5i(/*MonoBehaviour::.ctor*/b,0);
  return
}

// static void BaseRotacion::.cctor()
function Iqg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void BaseRotacion::Start()
function Jqg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void BaseRotacion::Update()
function Kqg(b,e){
  b=b|0;
  e=e|0;
  var f=0,g=0,h=0;
  if(!(a[1287104]|0)){
    c[677108]=DO(2)/*typeof(UnityEngine.Input)*/|0;
    c[678938]=FO(-2147483647)/*Component::GetComponent<Union>*/|0;
    a[1287104]=1
  }
  Lqg(/*BaseRotacion::Rotacion*/b,0);
  e=c[677108]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0)XP(e);
  if(v5i(/*Input::GetButtonUp*/0,2959512/*"Fire2"*/,0)|0)a[b+20>>0]=1;
  e=c[677108]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0)XP(e);
  if(u5i(/*Input::GetButton*/0,2959512/*"Fire2"*/,0)|0){
    f=g6i(/*Transform::get_childCount*/V5i(/*Component::get_transform*/b,0)|0,0)|0;
    g=b+20|0;
    if((f|0)>4&d[g>>0]){
      f=0;
      while(1){
        h=V5i(/*Component::get_transform*/b,0)|0;
        h=(f|0)<(g6i(/*Transform::get_childCount*/e6i(/*Transform::GetChild*/e6i(/*Transform::GetChild*/h,(g6i(/*Transform::get_childCount*/V5i(/*Component::get_transform*/b,0)|0,0)|0)+-1|0,0)|0,0,0)|0,0)|0);
        e=V5i(/*Component::get_transform*/b,0)|0;
        e=e6i(/*Transform::GetChild*/e6i(/*Transform::GetChild*/e,(g6i(/*Transform::get_childCount*/V5i(/*Component::get_transform*/b,0)|0,0)|0)+-1|0,0)|0,0,0)|0;
        if(!h)break;
        h=e6i(/*Transform::GetChild*/e,f,0)|0;
        c[(OBj(h,c[678938]|0)|0)+16>>2]=2959536/*"libre"*/;
        f=f+1|0
      }
      h=e6i(/*Transform::GetChild*/e,0,0)|0;
      Mqg(/*Union::desconectar*/OBj(h,c[678938]|0)|0,0)
    }
    a[g>>0]=0
  }
  return
}

// void BaseRotacion::Rotacion()
function Lqg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=$n,h=$n;
  if(!(a[1287112]|0)){
    c[677108]=DO(2)/*typeof(UnityEngine.Input)*/|0;
    a[1287112]=1
  }
  d=V5i(/*Component::get_transform*/b,0)|0;
  e=c[677108]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0)XP(e);
  h=fa(s5i(/*Input::GetAxis*/0,2959560/*"Rotacion"*/,0));
  f=fa(g[b+16>>2]);
  F6i(/*Transform::Rotate*/d,fa(0.0),fa(0.0),fa(-fa(fa(h*f)*fa(d0i(/*Time::get_deltaTime*/0,0)))),0);
  return
}
