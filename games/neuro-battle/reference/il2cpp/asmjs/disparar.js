// disparar (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: bala (+16), bulletOffset (+20), disparo (+32), valido (+33), shootSound (+36), source (+40)

// void disparar::.ctor()
function lrg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0;
  f=i;
  i=i+16|0;
  e=f;
  if(!(a[1287200]|0)){
    d=DO(13)/*typeof(UnityEngine.GameObject)*/|0;
    c[677118]=d;
    a[1287200]=1
  }
  else d=c[677118]|0;
  d=UO(d)|0;
  Z5i(/*GameObject::.ctor*/d,0);
  c[b+16>>2]=d;
  c[e+0>>2]=0;
  c[e+4>>2]=0;
  c[e+8>>2]=0;
  X$i(/*Vector3::.ctor*/e,fa(0.0),fa(.5),fa(0.0),0);
  d=b+20|0;
  c[d+0>>2]=c[e+0>>2];
  c[d+4>>2]=c[e+4>>2];
  c[d+8>>2]=c[e+8>>2];
  l5i(/*MonoBehaviour::.ctor*/b,0);
  i=f;
  return
}

// void disparar::Awake()
function mrg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287208]|0)){
    d=FO(-2147483644)/*Component::GetComponent<AudioSource>*/|0;
    c[678944]=d;
    a[1287208]=1
  }
  else d=c[678944]|0;
  c[b+40>>2]=OBj(b,d)|0;
  return
}

// void disparar::Start()
function nrg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void disparar::Update()
function org(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0,g=0,h=0,j=0,k=0,l=0,m=0,n=0,o=0;
  o=i;
  i=i+128|0;
  n=o+104|0;
  g=o;
  l=o+16|0;
  j=o+32|0;
  k=o+48|0;
  e=o+60|0;
  f=o+72|0;
  h=o+88|0;
  if(!(a[1287216]|0)){
    c[677108]=DO(2)/*typeof(UnityEngine.Input)*/|0;
    c[677118]=DO(13)/*typeof(UnityEngine.GameObject)*/|0;
    a[1287216]=1
  }
  ;
  c[l+0>>2]=0;
  c[l+4>>2]=0;
  c[l+8>>2]=0;
  if(a[b+33>>0]|0){
    d=c[677108]|0;
    if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
    if(y5i(/*Input::GetKeyUp*/0,303,0)|0)a[b+32>>0]=0;
    d=c[677108]|0;
    if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
    d=w5i(/*Input::GetKey*/0,303,0)|0;
    m=b+32|0;
    if((a[m>>0]^1)&255&(d&1)){
      o6i(/*Transform::get_rotation*/j,V5i(/*Component::get_transform*/b,0)|0,0);
      d=b+20|0;
      c[g+0>>2]=c[j+0>>2];
      c[g+4>>2]=c[j+4>>2];
      c[g+8>>2]=c[j+8>>2];
      c[g+12>>2]=c[j+12>>2];
      c[n+0>>2]=c[d+0>>2];
      c[n+4>>2]=c[d+4>>2];
      c[n+8>>2]=c[d+8>>2];
      X0i(/*Quaternion::op_Multiply*/k,0,g,n,0);
      c[l+0>>2]=c[k+0>>2];
      c[l+4>>2]=c[k+4>>2];
      c[l+8>>2]=c[k+8>>2];
      d=c[b+16>>2]|0;
      j6i(/*Transform::get_position*/e,V5i(/*Component::get_transform*/b,0)|0,0);
      c[g+0>>2]=c[e+0>>2];
      c[g+4>>2]=c[e+4>>2];
      c[g+8>>2]=c[e+8>>2];
      c[n+0>>2]=c[l+0>>2];
      c[n+4>>2]=c[l+4>>2];
      c[n+8>>2]=c[l+8>>2];
      b0i(/*Vector3::op_Addition*/f,0,g,n,0);
      o6i(/*Transform::get_rotation*/h,V5i(/*Component::get_transform*/b,0)|0,0);
      c[g+0>>2]=c[f+0>>2];
      c[g+4>>2]=c[f+4>>2];
      c[g+8>>2]=c[f+8>>2];
      c[n+0>>2]=c[h+0>>2];
      c[n+4>>2]=c[h+4>>2];
      c[n+8>>2]=c[h+8>>2];
      c[n+12>>2]=c[h+12>>2];
      d=P5i(/*Object::Instantiate*/0,d,g,n,0)|0;
      e=c[677118]|0;
      do if((d|0)!=0?(ZO(d,e)|0)==0:0){
        qN(n,c[(c[d>>2]|0)+28>>2]|0,e);
        if(!(a[n>>0]&1))d=n+1|0;
        else d=c[n+8>>2]|0;
        r=0;
        d=dc(1026/*invoke aN*/,d|0)|0;
        k=r;
        r=0;
        if(!(k&1)?(r=0,bc(3842/*invoke VM*/,d|0),k=r,r=0,!(k&1)):0){
          Zkk(n);
          break
        }
        b=hg()|0;
        Zkk(n);
        tg(b|0)
      }
      while(0);
      N7i(/*AudioSource::PlayOneShot*/c[b+40>>2]|0,c[b+36>>2]|0,fa(100.0),0);
      a[m>>0]=1
    }
  }
  i=o;
  return
}
