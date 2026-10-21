// BaseMovimiento (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: velocidadMovimiento (+16)

// void BaseMovimiento::.ctor()
function Dqg(a,b){
  a=a|0;
  b=b|0;
  g[a+16>>2]=fa(1.0);
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void BaseMovimiento::Start()
function Eqg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void BaseMovimiento::Update()
function Fqg(a,b){
  a=a|0;
  b=b|0;
  Gqg(/*BaseMovimiento::Movimiento*/a,0);
  return
}

// void BaseMovimiento::Movimiento()
function Gqg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0,h=0,j=0,k=0,l=0,m=$n,n=$n;
  l=i;
  i=i+48|0;
  k=l+36|0;
  f=l;
  h=l+12|0;
  j=l+24|0;
  if(!(a[1287096]|0)){
    c[677108]=DO(2)/*typeof(UnityEngine.Input)*/|0;
    a[1287096]=1
  }
  d=V5i(/*Component::get_transform*/b,0)|0;
  e=c[677108]|0;
  if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0)XP(e);
  n=fa(s5i(/*Input::GetAxis*/0,2959440/*"Horizontal"*/,0));
  m=fa(s5i(/*Input::GetAxis*/0,2959480/*"Vertical"*/,0));
  c[f+0>>2]=0;
  c[f+4>>2]=0;
  c[f+8>>2]=0;
  X$i(/*Vector3::.ctor*/f,n,m,fa(0.0),0);
  m=fa(g[b+16>>2]);
  c[k+0>>2]=c[f+0>>2];
  c[k+4>>2]=c[f+4>>2];
  c[k+8>>2]=c[f+8>>2];
  a0i(/*Vector3::op_Multiply*/h,0,k,m,0);
  m=fa(d0i(/*Time::get_deltaTime*/0,0));
  c[k+0>>2]=c[h+0>>2];
  c[k+4>>2]=c[h+4>>2];
  c[k+8>>2]=c[h+8>>2];
  a0i(/*Vector3::op_Multiply*/j,0,k,m,0);
  c[k+0>>2]=c[j+0>>2];
  c[k+4>>2]=c[j+4>>2];
  c[k+8>>2]=c[j+8>>2];
  B6i(/*Transform::Translate*/d,k,0);
  i=l;
  return
}
