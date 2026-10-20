using UnityEngine;

public class PowerUp : MonoBehaviour
{
	private HUD hud;

	private AudioSource aud;

	private bool activo = true;

	private void Awake()
	{
		aud = GetComponent<AudioSource>();
	}

	private void OnTriggerEnter2D(Collider2D other)
	{
		if ((other.tag == "Player") & activo & (other.GetComponent<PlatformerCharacter2D>().maxSpeed > 0f))
		{
			aud.Play();
			hud = GameObject.Find("Main Camera").GetComponent<HUD>();
			hud.increaseScore(1);
			activo = false;
			base.gameObject.renderer.enabled = false;
		}
	}
}
