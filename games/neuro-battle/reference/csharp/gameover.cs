// Reconstruido del asm.js (reference/il2cpp/asmjs/gameover.js); en la escena «game_over».
using UnityEngine;

public class gameover : MonoBehaviour
{
    public GUIStyle estilo;
    public Texture aTexture;

    void Start() { }

    void Update()
    {
        if (Input.GetKey(KeyCode.E))
            Application.LoadLevel(2);
        if (Input.GetKey(KeyCode.Escape))
            Application.LoadLevel(0);
    }

    void OnGUI()
    {
        float tiempo = 0f;
        GUI.DrawTexture(new Rect(0f, 0f, Screen.width, Screen.height), aTexture, ScaleMode.StretchToFill, true, 0f);
        Rect rect = new Rect(0f, 20f, Screen.width, 100f);
        tiempo = PlayerPrefs.GetFloat("tiempo");
        GUI.Label(rect, "TU PUNTAJE FUE DE " + tiempo.ToString("0") + " SEGUNDOS", estilo);
    }
}
