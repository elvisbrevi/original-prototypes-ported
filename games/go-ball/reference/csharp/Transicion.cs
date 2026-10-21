using UnityEngine;

public class Transicion : MonoBehaviour
{
	private GUITexture gt;

	public static float _alpha = 1f;

	public static bool FadeIn;

	public static bool TransicionRealizada;

	public string sigNivel;

	private void Start()
	{
		Bola.perdio = false;
		Bola.gano = false;
		Objetivo.fin = false;
		TransicionRealizada = false;
		FadeIn = false;
		gt = (GUITexture)base.gameObject.GetComponent(typeof(GUITexture));
	}

	public static void fadeIn(float alpha)
	{
		_alpha = alpha;
	}

	public static void irA(string destino)
	{
		FadeIn = true;
		if (TransicionRealizada)
		{
			FadeIn = false;
			Application.LoadLevel(destino);
		}
	}

	private void Update()
	{
		if (Objetivo.fin)
		{
			FadeIn = Objetivo.fin;
			if (TransicionRealizada)
			{
				FadeIn = false;
				Application.LoadLevel(sigNivel);
			}
		}
		if (Bola.perdio)
		{
			FadeIn = true;
			if (TransicionRealizada)
			{
				FadeIn = false;
				Application.LoadLevel(Application.loadedLevelName);
			}
		}
		if (FadeIn)
		{
			if (_alpha < 1f)
			{
				_alpha += Time.deltaTime * 0.5f;
				gt.color = new Color(0f, 0f, 0f, _alpha);
			}
			else
			{
				TransicionRealizada = true;
			}
		}
		else if (_alpha > 0f)
		{
			_alpha -= Time.deltaTime * 0.5f;
			gt.color = new Color(0f, 0f, 0f, _alpha);
		}
		else
		{
			TransicionRealizada = false;
		}
	}

	private void OnGUI()
	{
		GUI.color = new Color(0f, 0f, 0f, _alpha);
		GUI.DrawTexture(new Rect(0f, 0f, Screen.width, Screen.height), gt.texture);
	}
}
