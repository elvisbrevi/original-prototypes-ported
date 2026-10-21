// muerte_enemigo (Assembly-CSharp.dll): el asm.js de cada método, con las llamadas, los tipos, los métodos y los textos anotados.
// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64; fa = Math.fround.
// Campos: vida (+16), anim (+20)

// void muerte_enemigo::.ctor()
function Arg(a,b){
  a=a|0;
  b=b|0;
  g[a+16>>2]=fa(5.0);
  l5i(/*MonoBehaviour::.ctor*/a,0);
  return
}

// void muerte_enemigo::Start()
function Brg(b,d){
  b=b|0;
  d=d|0;
  if(!(a[1287256]|0)){
    d=FO(-2147483646)/*Component::GetComponent<Animator>*/|0;
    c[678940]=d;
    a[1287256]=1
  }
  else d=c[678940]|0;
  c[b+20>>2]=OBj(b,d)|0;
  return
}

// void muerte_enemigo::Update()
function Crg(a,b){
  a=a|0;
  b=b|0;
  if(fa(g[a+16>>2])<=fa(0.0))F5i(/*Object::Destroy*/0,W5i(/*Component::get_gameObject*/a,0)|0,0);
  return
}

// void muerte_enemigo::OnTriggerEnter2D(Collider2D other)
function Drg(b,d,e){
  b=b|0;
  d=d|0;
  e=e|0;
  if(!(a[1287264]|0)){
    c[677106]=DO(1)/*typeof(Aparicion)*/|0;
    a[1287264]=1
  }
  if(bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2960032/*"neurona"*/,0)|0){
    if(bjh(/*String::Contains*/W_i(/*Object::get_name*/d,0)|0,2960064/*"neurona_base"*/,0)|0){
      e=c[677106]|0;
      if((a[e+130>>0]|0)!=0?(c[e+152>>2]|0)==0:0){
        XP(e);
        e=c[677106]|0
      }
      Q6i(/*PlayerPrefs::SetFloat*/0,2959952/*"tiempo"*/,fa(g[c[e+64>>2]>>2]),0);
      Y3i(/*Application::LoadLevel*/0,3,0)
    }
    F5i(/*Object::Destroy*/0,W5i(/*Component::get_gameObject*/d,0)|0,0);
    F5i(/*Object::Destroy*/0,W5i(/*Component::get_gameObject*/b,0)|0,0)
  }
  return
}
