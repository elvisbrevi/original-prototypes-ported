using UnityEngine;

public class Destroyer : MonoBehaviour
{
	public float amount = 54.6f;

	private void OnTriggerEnter2D(Collider2D other)
	{
		if (other.tag == "Particulas")
		{
			Object.Destroy(other.gameObject);
		}
		if (other.tag == "Suelo")
		{
			Vector3 position = other.transform.position;
			position.x += amount;
			other.transform.position = position;
		}
		else if ((bool)other.gameObject.transform.parent)
		{
			Object.Destroy(other.gameObject.transform.parent.gameObject);
		}
		else
		{
			Object.Destroy(other.gameObject);
		}
	}
}
