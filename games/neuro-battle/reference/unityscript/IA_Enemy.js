// Reconstruido del asm.js (reference/il2cpp/asmjs/IA_Enemy.js): UnityScript, en el «enemigo».
// Las llamadas a CrearWaypoint y Seguir son virtuales (en UnityScript todo método lo es).
#pragma strict

enum EstadoEnemigo { quieto, huir, atacar, patrullar }

var target : Transform;
var waypoint : Transform;
var velocidadMovimiento_x : float = 3;
var velocidadMovimiento_y : float = 1.5;
var velocidadHuida : float = 4.5;
var estadoEnemigo : EstadoEnemigo;
var distanciaTarget : float = 10;
var distanciaWaypoint : float = 4;
var distanciaMuerte : float = 1.1;
var rangoMovimiento : float = 15;
var tiempoQuieto : int = 5;
private var tiempoQuietoCheck : int;

function Start() {
	estadoEnemigo = EstadoEnemigo.patrullar;
}

function CrearWaypoint() {
	var esfera = GameObject.CreatePrimitive(PrimitiveType.Sphere);
	waypoint = esfera.transform;
	(esfera.GetComponent(MeshRenderer) as MeshRenderer).enabled = false;
	waypoint.transform.position = Vector3(Random.Range(-66, 19), Random.Range(-32, 18), 0);
}

// Avanza hacia el objetivo por separado en x y en y. Mathf.Round quedó dentro de la función
// (roundf, que redondea las mitades hacia afuera): con menos de media unidad de diferencia, el
// signo de ±0 es 1 y el enemigo avanza hacia la derecha o hacia arriba.
function Seguir(objetivo : Transform) {
	var dx = Mathf.Sign(Mathf.Round(objetivo.position.x - transform.position.x));
	transform.position = Vector3(transform.position.x + dx * velocidadMovimiento_x * Time.deltaTime,
	                             transform.position.y, transform.position.z);
	var dy = Mathf.Sign(Mathf.Round(objetivo.position.y - transform.position.y));
	transform.position = Vector3(transform.position.x,
	                             transform.position.y + dy * velocidadMovimiento_y * Time.deltaTime,
	                             transform.position.z);
}

function Update() {
	var direccion = Vector3.zero;
	var distancia = Vector3.Distance(transform.position, target.position);
	if (estadoEnemigo != EstadoEnemigo.patrullar)
		waypoint != null;  // una comparación sin efecto
	if (distancia > distanciaTarget) {
		switch (estadoEnemigo) {
		case EstadoEnemigo.atacar:
			Atacar(distancia);
			break;
		case EstadoEnemigo.patrullar:
			if (waypoint == null)
				CrearWaypoint();
			// El waypoint no se renueva: al llegar a él, el enemigo se queda quieto.
			if (Vector3.Distance(transform.position, waypoint.position) > distanciaWaypoint)
				Seguir(waypoint);
			break;
		case EstadoEnemigo.huir:
			// Nadie pone este estado. Multiplica la posición entera por deltaTime.
			direccion = transform.position - target.position;
			direccion = direccion / direccion.magnitude;
			transform.position = Vector3(direccion.x + transform.position.x,
			                             direccion.y + transform.position.y, 0) * Time.deltaTime;
			break;
		case EstadoEnemigo.quieto:
			if (Time.time >= tiempoQuietoCheck) {
				tiempoQuietoCheck = Time.time + tiempoQuieto;
				estadoEnemigo = EstadoEnemigo.patrullar;
			}
			break;
		}
	} else {
		estadoEnemigo = EstadoEnemigo.atacar;
		Atacar(distancia);
	}
}

// En el asm.js es un bloque compartido por los dos caminos del switch.
function Atacar(distancia : float) {
	Seguir(target);
	if (distancia <= distanciaMuerte)
		DestroyObject(gameObject);
	if (distancia >= distanciaTarget)
		estadoEnemigo = EstadoEnemigo.quieto;
}
