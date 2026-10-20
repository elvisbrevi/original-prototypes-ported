using UnityEngine;

[RequireComponent(typeof(PlatformerCharacter2D))]
public class Platformer2DUserControl : MonoBehaviour
{
	private PlatformerCharacter2D character;

	private bool jump;

	private void Awake()
	{
		character = GetComponent<PlatformerCharacter2D>();
	}

	private void Update()
	{
		try
		{
			if (Input.GetMouseButtonDown(0) & (Time.timeScale > 0f))
			{
				jump = true;
			}
		}
		catch
		{
		}
	}

	private void FixedUpdate()
	{
		character.Move(1f, crouch: false, jump);
		jump = false;
	}
}
