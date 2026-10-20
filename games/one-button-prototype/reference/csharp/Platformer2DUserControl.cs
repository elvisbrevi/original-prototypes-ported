using UnityEngine;

namespace UnityStandardAssets._2D;

[RequireComponent(typeof(PlatformerCharacter2D))]
public class Platformer2DUserControl : MonoBehaviour
{
	public PlatformerCharacter2D m_Character;

	private bool m_Jump;

	private float h = 1f;

	private void Awake()
	{
		m_Character = GetComponent<PlatformerCharacter2D>();
	}

	private void Update()
	{
		if (!m_Jump && Input.GetMouseButtonDown(0))
		{
			m_Jump = true;
		}
	}

	private void muere()
	{
		Application.LoadLevel(Application.loadedLevel);
	}

	private void OnTriggerEnter2D(Collider2D other)
	{
		if (other.name.Contains("Final"))
		{
			Debug.Log("Fin");
			GameObject.Find("Main Camera").GetComponent<GUI_Prototipo>().pauseGame = true;
		}
		if (other.name.Contains("Spike"))
		{
			muere();
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
		Collider2D collider = coll.collider;
		if (coll.gameObject.name.Contains("Dragon") || coll.gameObject.name.Contains("Dragon"))
		{
			Vector3 center = collider.bounds.center;
			if (m_Character.m_GroundCheck.position.y > center.y)
			{
				Object.DestroyObject(coll.gameObject);
			}
			else
			{
				muere();
			}
		}
	}

	private void FixedUpdate()
	{
		bool key = Input.GetKey(KeyCode.LeftControl);
		m_Character.Move(h, key, m_Jump);
		m_Jump = false;
	}
}
