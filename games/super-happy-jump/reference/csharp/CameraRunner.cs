using UnityEngine;

public class CameraRunner : MonoBehaviour
{
	public Transform player;

	public float y;

	private static float x = 5f;

	private static float max;

	private static float min;

	private static float velocidadShake = 0.5f;

	private static bool ida;

	public static bool isShake;

	private void Start()
	{
		max = x + 1f;
		min = x - 1f;
		ida = false;
		isShake = false;
		velocidadShake = 0.5f;
		x = 4f;
	}

	private void Update()
	{
		base.transform.position = new Vector3(player.position.x + x, y, -10f);
		if (isShake && velocidadShake > 0f)
		{
			shake();
		}
		else
		{
			isShake = false;
		}
	}

	private static void shake()
	{
		if (ida)
		{
			if (x < max)
			{
				x += velocidadShake * Time.timeScale;
			}
			else
			{
				ida = false;
			}
		}
		else if (x > min)
		{
			x -= velocidadShake * Time.timeScale;
		}
		else
		{
			ida = true;
		}
		max -= 0.2f * Time.deltaTime;
		min += 0.2f * Time.deltaTime;
		velocidadShake -= 0.02f * Time.timeScale;
	}
}
