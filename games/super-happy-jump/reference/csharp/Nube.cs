using UnityEngine;

public class Nube : MonoBehaviour
{
	public float velocidad;

	private void Start()
	{
	}

	private void Update()
	{
		base.transform.position = new Vector3(base.transform.position.x - velocidad * Time.deltaTime, base.transform.position.y, base.transform.position.z);
		if (base.transform.position.x + 11f < Camera.main.transform.position.x)
		{
			base.transform.position = new Vector3(Camera.main.transform.position.x + 11f, base.transform.position.y, base.transform.position.z);
		}
	}
}
