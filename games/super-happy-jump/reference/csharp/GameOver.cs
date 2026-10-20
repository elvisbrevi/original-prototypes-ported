using UnityEngine;

public class GameOver : MonoBehaviour
{
	private int score;

	private void Start()
	{
		score = PlayerPrefs.GetInt("Score");
	}

	private void OnGUI()
	{
		GUI.Label(new Rect(Screen.width / 2 - 40, 50f, 80f, 30f), "GAME OVER");
		GUI.Label(new Rect(Screen.width / 2 - 40, 300f, 80f, 30f), "Score: " + score);
		if (GUI.Button(new Rect(Screen.width / 2 - 30, 350f, 60f, 30f), "Retry?"))
		{
			Application.LoadLevel(0);
		}
	}
}
