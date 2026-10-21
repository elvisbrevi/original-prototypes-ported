using UnityEngine;

public class Bola : MonoBehaviour
{
	public static Vector2 posicion;

	public int limiteMin;

	public int limiteMax;

	public static bool perdio;

	public static bool gano;

	private float velocidad = 2f;

	private float velocidadSalto = 4f;

	private bool saltando;

	private bool colision;

	private bool exploto;

	private bool rodo;

	public static bool Activo;

	public AudioSource explosion;

	private void Start()
	{
		Activo = true;
	}

	private void Update()
	{
		posicion = base.transform.position;
		moverse();
		perder();
	}

	private void OnCollisionStay(Collision collision)
	{
		colision = true;
		if (collision.gameObject.name.Contains("Top"))
		{
			saltando = false;
		}
		if (collision.gameObject.name == "Sierra")
		{
			if (!exploto)
			{
				explosion.Play();
				exploto = true;
			}
			Activo = false;
			base.particleEmitter.emit = true;
		}
		if (collision.gameObject.name == "Explosivo1")
		{
			base.rigidbody.AddForce(5f, 0f, 0f, ForceMode.Impulse);
		}
	}

	private void OnParticleCollision(GameObject other)
	{
		if (!exploto)
		{
			explosion.Play();
			exploto = true;
		}
		Activo = false;
		base.particleEmitter.emit = true;
	}

	private void moverse()
	{
		base.rigidbody.AddForce(Input.GetAxis("Horizontal") * velocidad, 0f, 0f, ForceMode.Acceleration);
		if (Input.GetButtonDown("Saltar") && colision && !saltando)
		{
			base.rigidbody.velocity = new Vector3(base.rigidbody.velocity.x, velocidadSalto);
			saltando = true;
		}
	}

	private void perder()
	{
		if (base.particleEmitter.particleCount > 1)
		{
			base.gameObject.renderer.enabled = false;
		}
		if (base.rigidbody.position.y < (float)limiteMin || base.rigidbody.position.y > (float)limiteMax || (!base.gameObject.renderer.enabled && base.particleEmitter.particleCount == 0))
		{
			Object.Destroy(base.gameObject);
			if (!gano)
			{
				perdio = true;
			}
		}
	}
}
