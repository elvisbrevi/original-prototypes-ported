using UnityEngine;

public class Camara : MonoBehaviour
{
	private const float borde = 2.5f;

	private const float ancho = 15f;

	private GUITexture gt;

	public GUISkin skin;

	public bool SeguirY;

	public bool SeguirX;

	private Rect menuRect;

	private bool salir;

	private bool irMenu;

	private bool reiniciar;

	private bool repetir;

	public Rect windowRect;

	public AudioSource click;

	private float anteriorTime;

	private void Start()
	{
		gt = (GUITexture)base.gameObject.GetComponent(typeof(GUITexture));
		menuRect = new Rect(Screen.width - 80, 0f, 80f, 20f);
		windowRect = new Rect(Screen.width / 2 - 125, Screen.height / 2, 250f, 50f);
		if (Application.isWebPlayer)
		{
			base.transform.position = new Vector3(base.transform.position.x, base.transform.position.y, base.transform.position.z * 1.2f);
		}
	}

	private void Update()
	{
		if (irMenu)
		{
			Transicion.irA("menuPrincipal");
		}
		if (repetir)
		{
			Transicion.irA(Application.loadedLevelName);
		}
		if (SeguirY && Bola.Activo)
		{
			base.transform.position = new Vector3(base.transform.position.x, Bola.posicion.y, base.transform.position.z);
		}
		if (SeguirX && Bola.Activo)
		{
			base.transform.position = new Vector3(Bola.posicion.x, base.transform.position.y, base.transform.position.z);
		}
	}

	private void OnGUI()
	{
		GUI.skin = skin;
		GUI.DrawTexture(menuRect, gt.texture);
		botonSalir();
		if (Screen.fullScreen)
		{
			if (GUI.Button(new Rect(menuRect.xMax - 35f, 2.5f, 15f, 15f), new GUIContent(string.Empty, "Ventana"), skin.FindStyle("maximizar")))
			{
				if (!click.isPlaying)
				{
					click.Play();
				}
				Screen.fullScreen = false;
			}
		}
		else if (GUI.Button(new Rect(menuRect.xMax - 35f, 2.5f, 15f, 15f), new GUIContent(string.Empty, "Maximizar"), skin.FindStyle("maximizar")))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Screen.fullScreen = true;
		}
		if (GUI.Button(new Rect(menuRect.xMax - 55f, 2.5f, 15f, 15f), new GUIContent(string.Empty, "Opciones"), skin.FindStyle("opciones")) && !click.isPlaying)
		{
			click.Play();
		}
		botonReiniciar();
		GUI.Label(new Rect(menuRect.x, menuRect.y + menuRect.height + 2.5f, menuRect.width, menuRect.height), GUI.tooltip, skin.FindStyle("toolTip"));
	}

	private void botonReiniciar()
	{
		if (GUI.Button(new Rect(menuRect.xMax - 75f, 2.5f, 15f, 15f), new GUIContent(string.Empty, "Reiniciar"), skin.FindStyle("reiniciar")))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			reiniciar = true;
		}
		if (reiniciar)
		{
			windowRect = GUI.Window(1, windowRect, VentanaReiniciar, "¿Desea volver a iniciarla partida?");
		}
	}

	private void botonSalir()
	{
		if (GUI.Button(new Rect(menuRect.xMax - 17.5f, 2.5f, 15f, 15f), new GUIContent(string.Empty, "Salir"), skin.FindStyle("cerrar")))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			salir = true;
		}
		if (salir)
		{
			windowRect = GUI.Window(0, windowRect, VentanaSalir, "¿Desea salir a la pantalla de inicio?");
		}
	}

	private void VentanaReiniciar(int windowID)
	{
		Transicion.fadeIn(0.7f);
		if (Time.timeScale != 0f)
		{
			anteriorTime = Time.timeScale;
		}
		Time.timeScale = 0f;
		if (GUI.Button(new Rect(10f, 20f, 100f, 20f), "Si"))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Time.timeScale = anteriorTime;
			repetir = true;
			reiniciar = false;
		}
		if (GUI.Button(new Rect(140f, 20f, 100f, 20f), "No"))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Time.timeScale = anteriorTime;
			reiniciar = false;
		}
	}

	private void VentanaSalir(int windowID)
	{
		Transicion.fadeIn(0.7f);
		if (Time.timeScale != 0f)
		{
			anteriorTime = Time.timeScale;
		}
		Time.timeScale = 0f;
		if (GUI.Button(new Rect(10f, 20f, 100f, 20f), "Si"))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Time.timeScale = anteriorTime;
			irMenu = true;
			salir = false;
		}
		if (GUI.Button(new Rect(140f, 20f, 100f, 20f), "No"))
		{
			if (!click.isPlaying)
			{
				click.Play();
			}
			Time.timeScale = anteriorTime;
			salir = false;
		}
	}
}
