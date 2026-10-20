using UnityEngine;

public class GUI_Prototipo : MonoBehaviour
{
	public bool pauseGame;

	private bool showGUI;

	private int cantidadNiveles = 4;

	private float anchoBoton = 100f;

	public Rect windowRect = default(Rect);

	private void Update()
	{
		if (Input.GetKeyDown("p"))
		{
			pauseGame = !pauseGame;
		}
		if (pauseGame)
		{
			Time.timeScale = 0f;
			pauseGame = true;
			showGUI = true;
		}
		if (!pauseGame)
		{
			Time.timeScale = 1f;
			pauseGame = false;
			showGUI = false;
		}
	}

	private void OnGUI()
	{
		if (showGUI)
		{
			float num = anchoBoton * (float)(cantidadNiveles / 2) + 20f;
			float num2 = anchoBoton * 2f + 40f;
			windowRect = new Rect((float)(Screen.width / 2) - num / 2f, (float)(Screen.height / 2) - num2 / 2f, num, num2);
			windowRect = GUI.Window(0, windowRect, CrearWindow, "Nivel");
		}
	}

	private void CrearWindow(int windowID)
	{
		int num = 1;
		for (int i = 0; i < 2; i++)
		{
			for (int j = 0; j < cantidadNiveles / 2; j++)
			{
				if (GUI.Button(new Rect(10f + (float)j * anchoBoton, 30f + (float)i * anchoBoton, anchoBoton, anchoBoton), num.ToString()))
				{
					Application.LoadLevel(num - 1);
				}
				num++;
			}
		}
	}
}
