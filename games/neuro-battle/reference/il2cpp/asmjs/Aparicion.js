// Aparicion (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: timerMax (+16), timer (+20), neuronas (+24), zoom (+28), static contadorTiempo

// void Aparicion::.ctor()
function zqg(a,b){
  a=a|0;
  b=b|0;
  g[a+16>>2]=fa(1.0);
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// static void Aparicion::.cctor()
function Aqg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void Aparicion::Start()
function Bqg(a,b){
  a=a|0;
  b=b|0;
  return
}

// void Aparicion::Update()
function Cqg(b,d){
  b=b|0;
  d=d|0;
  var e=0,f=0,h=0,j=0,k=0,l=0,m=0,n=0,o=0,p=0,q=0,s=$n,t=$n;
  p=i;
  i=i+64|0;
  l=p+40|0;
  j=p;
  h=p+12|0;
  k=p+24|0;
  if(!(a[1287088]|0)){
    c[677106]=DO(1)/*typeof(Aparicion)*/|0;
    c[677108]=DO(2)/*typeof(UnityEngine.Input)*/|0;
    c[677110]=DO(3)/*typeof(UnityEngine.UnityException)*/|0;
    a[1287088]=1
  }
  d=c[677106]|0;
  if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0){
    XP(d);
    d=c[677106]|0
  }
  s=fa(g[c[d+64>>2]>>2]);
  s=fa(s+fa(d0i(/*Time::get_deltaTime*/0,0)));
  g[c[(c[677106]|0)+64>>2]>>2]=s;
  d=c[677108]|0;
  if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
  if(w5i(/*Input::GetKey*/0,27,0)|0)Y3i(/*Application::LoadLevel*/0,0,0);
  d=c[677108]|0;
  if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
  if(y5i(/*Input::GetKeyUp*/0,101,0)|0)a[b+28>>0]=0;
  d=c[677108]|0;
  if((a[d+130>>0]|0)!=0?(c[d+152>>2]|0)==0:0)XP(d);
  f=w5i(/*Input::GetKey*/0,101,0)|0;
  d=b+28|0;
  if((a[d>>0]^1)&255&(f&1)){
    f=fa(i4i(/*Camera::get_orthographicSize*/s4i(/*Camera::get_main*/0,0)|0,0))==fa(9.0);
    e=s4i(/*Camera::get_main*/0,0)|0;
    if(f)j4i(/*Camera::set_orthographicSize*/e,fa(20.0),0);
    else j4i(/*Camera::set_orthographicSize*/e,fa(9.0),0);
    a[d>>0]=1
  }
  f=b+20|0;
  s=fa(g[f>>2]);
  s=fa(s+fa(d0i(/*Time::get_deltaTime*/0,0)));
  g[f>>2]=s;
  if(s>fa(g[b+16>>2])){
    d=c[b+24>>2]|0;
    r=0;
    e=Ta(727/*invoke Random::Range*/,0,0,19,0)|0;
    b=r;
    r=0;
    if(!(((((!(b&1)?(r=0,m=Ta(727/*invoke Random::Range*/,0,-66,19,0)|0,b=r,r=0,!(b&1)):0)?(r=0,n=Ta(727/*invoke Random::Range*/,0,-32,18,0)|0,b=r,r=0,!(b&1)):0)?(c[h+0>>2]=0,c[h+4>>2]=0,c[h+8>>2]=0,t=fa(m|0),s=fa(n|0),r=0,ec(1/*invoke Vector3::.ctor*/,h|0,+t,+s,0.0,0),n=r,r=0,!(n&1)):0)?(r=0,Xb(1242/*invoke Quaternion::get_identity*/,k|0,0,0),n=r,r=0,!(n&1)):0)?(n=c[d+16+(ca(c[(c[d>>2]|0)+96>>2]|0,e)|0)>>2]|0,r=0,c[j+0>>2]=c[h+0>>2],c[j+4>>2]=c[h+4>>2],c[j+8>>2]=c[h+8>>2],c[l+0>>2]=c[k+0>>2],c[l+4>>2]=c[k+4>>2],c[l+8>>2]=c[k+8>>2],c[l+12>>2]=c[k+12>>2],mb(296/*invoke Object::Instantiate*/,0,n|0,j|0,l|0,0)|0,n=r,r=0,!(n&1)):0))o=31;
    do if((o|0)==31){
      d=hg(485008)|0;
      o=G;
      if((o|0)==(Kh(485008)|0)){
        d=tl(d|0)|0;
        r=0;
        e=Qb(3301/*invoke nM*/,c[677110]|0,c[c[d>>2]>>2]|0)|0;
        o=r;
        r=0;
        if(!(o&1)){
          if(e){
            Sj();
            break
          }
          p=im(4)|0;
          c[p>>2]=c[d>>2];
          r=0;
          Xb(1815/*invoke Enumerator<object, byte>::System.Collections.IDictionaryEnumerator.get_Entry*/,p|0,485008,0);
          p=r;
          r=0
        }
        d=hg()|0;
        r=0;
        gb(7/*invoke ICk*/);
        p=r;
        r=0;
        if(p&1){
          p=hg(0)|0;
          kR(p)
        }
        else q=d
      }
      else q=d;
      tg(q|0)
    }
    while(0);
    g[f>>2]=fa(0.0)
  }
  i=p;
  return
}
