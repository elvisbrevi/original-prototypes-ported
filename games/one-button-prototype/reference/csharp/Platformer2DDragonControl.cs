using UnityEngine;

namespace UnityStandardAssets._2D;

[RequireComponent(typeof(PlatformerDragon2D))]
public class Platformer2DDragonControl : MonoBehaviour
{
	private PlatformerDragon2D m_Character;

	private float h = 1f;

	private void Awake()
	{
		m_Character = GetComponent<PlatformerDragon2D>();
	}

	private void Update()
	{
	}

	private void OnTriggerEnter2D(Collider2D other)
	{
		if (other.name.Contains("Spike"))
		{
			h = 1f;
			Object.Destroy(this);
		}
	}

	private void OnCollisionEnter2D(Collision2D coll)
	{
		if (coll.gameObject.name.Contains("ParedRight"))
		{
			h = -1f;
		}
		else if (coll.gameObject.name.Contains("ParedLeft"))
		{
			h = 1f;
		}
	}

	private void FixedUpdate()
	{
		m_Character.Move(h, crouch: false, jump: false);
	}
}
