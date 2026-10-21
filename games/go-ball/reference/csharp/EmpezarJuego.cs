using UnityEngine;

public class EmpezarJuego : MonoBehaviour
{
	public GUISkin skin;

	public AudioSource click;

	private void OnGUI()
	{
		GUI.skin = skin;
		GUI.color = new Color(1f, 1f, 1f, 1f - Transicion._alpha);
		GUI.Label(new Rect(100f, 300f, 10f, 10f), "Go Ball", skin.FindStyle("titulo"));
		if (GUI.Button(new Rect(100f, 370f, 100f, 20f), "Jugar"))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Objetivo.fin = true;
		}
		if (GUI.Button(new Rect(100f, 395f, 100f, 20f), "Salir"))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Application.Quit();
		}
	}
}
