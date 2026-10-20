using UnityEngine;

public class HUD : MonoBehaviour
{
	public static ESTADO_GUI estadoStatic;

	private ESTADO_GUI estadoGUI;

	private int playerScore;

	private int bestPlayerScore;

	public GUISkin guiSkin;

	private bool newRecord;

	private AudioSource aud;

	public Rect windowRectFin;

	private void Awake()
	{
		aud = GetComponent<AudioSource>();
	}

	private void Start()
	{
		bestPlayerScore = PlayerPrefs.GetInt("BestScore");
		Time.timeScale = 0f;
		windowRectFin = new Rect(Screen.width / 2 - 320, Screen.height / 2 - 300, 640f, 480f);
	}

	private void Update()
	{
		estadoStatic = estadoGUI;
	}

	public void increaseScore(int amount)
	{
		playerScore += amount;
	}

	private void OnGUI()
	{
		if (!CameraRunner.isShake && !Spawn.activo)
		{
			estadoGUI = ESTADO_GUI.ShareGUI;
		}
		if (estadoGUI == ESTADO_GUI.MainGUI)
		{
			GUI.Label(new Rect(Screen.width / 2 - 140, 100f, 280f, 400f), string.Empty, guiSkin.GetStyle("titulo"));
			if (GUI.Button(new Rect(Screen.width - 150, Screen.height - 130, 120f, 100f), string.Empty, guiSkin.GetStyle("button_play")))
			{
				aud.Play();
				estadoGUI = ESTADO_GUI.InGameGUI;
				Time.timeScale = 1f;
			}
		}
		else if (estadoGUI == ESTADO_GUI.InGameGUI)
		{
			GUI.Label(new Rect(10f, 100f, 100f, 40f), playerScore.ToString(), guiSkin.GetStyle("GUI"));
		}
		else if (estadoGUI == ESTADO_GUI.ShareGUI)
		{
			windowRectFin = GUI.Window(0, new Rect(Screen.width / 2 - 250, 100f, 500f, 400f), ventanaFin, string.Empty, guiSkin.GetStyle("ventana"));
			if (GUI.Button(new Rect(Screen.width - 150, Screen.height - 130, 120f, 100f), string.Empty, guiSkin.GetStyle("button_play")))
			{
				aud.Play();
				Spawn.activo = true;
				Application.LoadLevel(0);
			}
		}
	}

	private void ventanaFin(int windowID)
	{
		GUI.Label(new Rect(windowRectFin.width / 2f - 256f, 0f, 512f, 112f), "Game Over", guiSkin.GetStyle("base_titulo"));
		if (playerScore > bestPlayerScore)
		{
			newRecord = true;
			bestPlayerScore = playerScore;
			PlayerPrefs.SetInt("BestScore", playerScore);
		}
		if (newRecord)
		{
			GUI.Label(new Rect(windowRectFin.width / 2f - 175f, 150f, 350f, 100f), "New Record!", guiSkin.GetStyle("label_record"));
			GUI.Label(new Rect(windowRectFin.width / 2f - 225f, 270f, 450f, 100f), "Score: " + playerScore, guiSkin.GetStyle("font_default"));
		}
		else
		{
			GUI.Label(new Rect(windowRectFin.width / 2f - 225f, 150f, 450f, 100f), "Score: " + playerScore, guiSkin.GetStyle("font_default"));
			GUI.Label(new Rect(windowRectFin.width / 2f - 200f, 250f, 400f, 70f), "Best: " + bestPlayerScore, guiSkin.GetStyle("font_default"));
		}
	}
}
