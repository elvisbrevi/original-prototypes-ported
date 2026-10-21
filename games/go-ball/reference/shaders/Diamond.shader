Shader "FX/Diamond" {
Properties {
 _Color ("Color", Color) = (1,1,1,1)
 _ReflectTex ("Reflection Texture", CUBE) = "dummy.jpg" { TexGen CubeReflect }
 _RefractTex ("Refraction Texture", CUBE) = "dummy.jpg" { TexGen CubeReflect }
}
SubShader { 
 Tags { "QUEUE"="Transparent" }
 Pass {
  Tags { "QUEUE"="Transparent" }
  Color (0,0,0,0)
  ZWrite Off
  Cull Front
  Offset -1, -1
  SetTexture [_RefractTex] { ConstantColor [_Color] combine texture * constant, primary alpha }
  SetTexture [_ReflectTex] { combine previous, previous alpha +- texture alpha }
 }
 Pass {
  Tags { "QUEUE"="Transparent" }
  Fog {
   Color (0,0,0,0)
  }
  Blend One One
  SetTexture [_RefractTex] { ConstantColor [_Color] combine texture * constant }
  SetTexture [_ReflectTex] { combine texture + previous, previous alpha +- texture alpha }
 }
}
SubShader { 
 Pass {
  Color (0,0,0,0)
  Cull Front
  SetTexture [_RefractTex] { ConstantColor [_Color] combine texture * constant, primary alpha }
 }
 Pass {
  Fog {
   Color (0,0,0,0)
  }
  Blend One One
  SetTexture [_RefractTex] { ConstantColor [_Color] combine texture * constant }
 }
}
SubShader { 
 Pass {
  Color [_Color]
 }
}
}