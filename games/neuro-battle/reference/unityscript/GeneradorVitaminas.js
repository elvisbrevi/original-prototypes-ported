// Reconstruido del asm.js (reference/il2cpp/asmjs/GeneradorVitaminas.js): UnityScript, en los 17
// generadores del nivel.
#pragma strict

var objetos : GameObject[];
var tiempoLanzar : int = 1;
private var tiempoLanzarCheck : int;

function Update() {
	// Time.time se compara con un entero y la suma se trunca al guardarla.
	if (Time.time >= tiempoLanzarCheck) {
		Instantiate(objetos[Random.Range(0, objetos.length)], transform.position, Quaternion.identity);
		tiempoLanzarCheck = Time.time + tiempoLanzar;
	}
}
