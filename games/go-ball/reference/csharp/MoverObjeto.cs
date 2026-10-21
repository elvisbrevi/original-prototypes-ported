using UnityEngine;

public class MoverObjeto : MonoBehaviour
{
	private Vector3 screeSpace;

	private Vector3 offset;

	public bool Horizontal;

	public bool Vertical;

	public bool Auto;

	public bool Rotacion;

	public float MinVertical;

	public float MaxVertical;

	public float MinHorizontal;

	public float MaxHorizontal;

	public float MinRotar;

	public float MaxRotar;

	public float VelocidadHorizontal;

	public float VelocidadVertical;

	public float VelocidadRotacion;

	public bool idaVertical;

	public bool idaHorizontal;

	public bool idaRotar;

	private void Update()
	{
		MoverAutomaticamente();
	}

	private void OnMouseDown()
	{
		screeSpace = Camera.main.WorldToScreenPoint(base.transform.position);
		if (Horizontal)
		{
			offset = base.transform.position - Camera.main.ScreenToWorldPoint(new Vector3(Input.mousePosition.x, screeSpace.y, screeSpace.z));
		}
		if (Vertical)
		{
			offset = base.transform.position - Camera.main.ScreenToWorldPoint(new Vector3(screeSpace.x, Input.mousePosition.y, screeSpace.z));
		}
	}

	private void OnMouseDrag()
	{
		if (!Auto)
		{
			MoverHorizontalmente();
			MoverVerticalmente();
		}
	}

	private void MoverAutomaticamente()
	{
		if (!Auto)
		{
			return;
		}
		if (Horizontal)
		{
			if (base.transform.position.x < MaxHorizontal && idaHorizontal)
			{
				base.transform.position = new Vector3(base.transform.position.x + VelocidadHorizontal * Time.deltaTime, base.transform.position.y, base.transform.position.z);
			}
			else
			{
				idaHorizontal = false;
			}
			if (base.transform.position.x > MinHorizontal && !idaHorizontal)
			{
				base.transform.position = new Vector3(base.transform.position.x - VelocidadHorizontal * Time.deltaTime, base.transform.position.y, base.transform.position.z);
			}
			else
			{
				idaHorizontal = true;
			}
		}
		if (Vertical)
		{
			if (base.transform.position.y < MaxVertical && idaVertical)
			{
				base.transform.position = new Vector3(base.transform.position.x, base.transform.position.y + VelocidadVertical * Time.deltaTime, base.transform.position.z);
			}
			else
			{
				idaVertical = false;
			}
			if (base.transform.position.y > MinVertical && !idaVertical)
			{
				base.transform.position = new Vector3(base.transform.position.x, base.transform.position.y - VelocidadVertical * Time.deltaTime, base.transform.position.z);
			}
			else
			{
				idaVertical = true;
			}
		}
		if (Rotacion)
		{
			if (base.transform.position.y < MaxHorizontal && idaRotar)
			{
				base.transform.Rotate(new Vector3(0f, 0f, Time.deltaTime) * VelocidadRotacion);
			}
			else
			{
				idaRotar = false;
			}
			if (base.transform.position.y > MinHorizontal && !idaRotar)
			{
				base.transform.Rotate(new Vector3(0f, 0f, Time.deltaTime) * (0f - VelocidadRotacion));
			}
			else
			{
				idaRotar = true;
			}
		}
	}

	private void MoverHorizontalmente()
	{
		if (Horizontal)
		{
			Vector3 position = new Vector3(Input.mousePosition.x, screeSpace.y, screeSpace.z);
			Vector3 vector = Camera.main.ScreenToWorldPoint(position) + offset;
			if (base.transform.position.x < vector.x - 0.1f && base.transform.position.x < MaxHorizontal)
			{
				base.transform.position = new Vector3(base.transform.position.x + 0.05f, base.transform.position.y, base.transform.position.z);
			}
			else if (base.transform.position.x > vector.x + 0.1f && base.transform.position.x > MinHorizontal)
			{
				base.transform.position = new Vector3(base.transform.position.x - 0.05f, base.transform.position.y, base.transform.position.z);
			}
		}
	}

	private void MoverVerticalmente()
	{
		if (Vertical)
		{
			Vector3 position = new Vector3(screeSpace.x, Input.mousePosition.y, screeSpace.z);
			Vector3 vector = Camera.main.ScreenToWorldPoint(position) + offset;
			if (base.transform.position.y < vector.y - 0.1f && base.transform.position.y < MaxVertical)
			{
				base.transform.position = new Vector3(base.transform.position.x, base.transform.position.y + 0.05f, base.transform.position.z);
			}
			else if (base.transform.position.y > vector.y + 0.1f && base.transform.position.y > MinVertical)
			{
				base.transform.position = new Vector3(base.transform.position.x, base.transform.position.y - 0.05f, base.transform.position.z);
			}
		}
	}
}
