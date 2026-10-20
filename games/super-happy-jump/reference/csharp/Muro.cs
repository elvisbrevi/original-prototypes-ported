using UnityEngine;

public class Muro : MonoBehaviour
{
	private AudioSource aud;

	private void Awake()
	{
		aud = GetComponent<AudioSource>();
	}

	private void OnTriggerEnter2D(Collider2D other)
	{
		if (other.tag == "Player" && other.GetComponent<PlatformerCharacter2D>().maxSpeed > 0f)
		{
			CameraRunner.isShake = true;
			aud.Play();
			other.GetComponent<PlatformerCharacter2D>().maxSpeed = 0f;
			other.GetComponent<PlatformerCharacter2D>().anim.SetBool("Ground", value: true);
			other.GetComponent<PlatformerCharacter2D>().anim.SetBool("Dead", value: true);
			Spawn.activo = false;
		}
	}
}
