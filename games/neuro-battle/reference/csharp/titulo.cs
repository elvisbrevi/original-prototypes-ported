// Reconstruido del asm.js (reference/il2cpp/asmjs/titulo.js); en la cámara de la escena «titulo».
using UnityEngine;

public class titulo : MonoBehaviour
{
    public Texture aTexture;

    void Start() { }

    void Update()
    {
        if (Input.GetKey(KeyCode.E))
            Application.LoadLevel(1);
        if (Input.GetKey(KeyCode.Escape))
            Application.Quit();
    }

    void OnGUI()
    {
        GUI.DrawTexture(new Rect(0f, 0f, Screen.width, Screen.height), aTexture, ScaleMode.StretchToFill, true, 0f);
    }
}
