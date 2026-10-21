// Reconstruido del asm.js (reference/il2cpp/asmjs/intrucciones.js); en la escena «instrucciones».
using UnityEngine;

public class intrucciones : MonoBehaviour
{
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
        GUI.DrawTexture(new Rect(0f, 0f, Screen.width, Screen.height), aTexture, ScaleMode.StretchToFill, true, 0f);
    }
}
