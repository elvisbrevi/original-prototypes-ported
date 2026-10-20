using UnityEngine;

public class Spawn : MonoBehaviour
{
	public GameObject muro;

	public float spawnMin = 1f;

	public float spawnMax = 2f;

	public float yMin;

	public float yMax;

	public static bool activo = true;

	private void Start()
	{
		Spawn_();
	}

	private void Spawn_()
	{
		Object.Instantiate(muro, new Vector3(base.transform.position.x, Random.Range(yMin, yMax), 0f), Quaternion.identity);
		if (activo)
		{
			Invoke("Spawn_", Random.Range(spawnMin, spawnMax));
		}
	}
}
