using UnityEngine;

public class Objetivo : MonoBehaviour
{
	public static bool fin;

	public AudioSource sonidoFinal;

	private void Start()
	{
		iTween.RotateBy(base.gameObject, iTween.Hash("x", 0.25, "easeType", "easeInOutBack", "loopType", "pingPong", "delay", 0.4));
	}

	private void Update()
	{
		if (!sonidoFinal.isPlaying && Bola.gano)
		{
			sonidoFinal.Play();
		}
		if (base.particleEmitter.particleCount > 1)
		{
			base.gameObject.renderer.enabled = false;
		}
		if (!base.gameObject.renderer.enabled && base.particleEmitter.particleCount == 0)
		{
			Object.Destroy(base.gameObject);
			fin = true;
		}
	}

	private void OnCollisionEnter(Collision collision)
	{
		if (collision.gameObject.name == "Player" && !collision.gameObject.particleEmitter.emit && Bola.Activo)
		{
			base.particleEmitter.emit = true;
			collision.gameObject.particleEmitter.emit = true;
			Bola.gano = true;
		}
	}
}
