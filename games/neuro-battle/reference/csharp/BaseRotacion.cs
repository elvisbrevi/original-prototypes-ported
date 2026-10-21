// Reconstruido del asm.js (reference/il2cpp/asmjs/BaseRotacion.js); en el «centro» de neurona_base.
using UnityEngine;

public class BaseRotacion : MonoBehaviour
{
    public float velocidadRotacion = 1f;
    // Lo ponen Union.Update (false) y Union.OnTriggerEnter2D (true); nadie lo lee
    // (animacionUnion.Update solo inicializa la clase).
    public static bool choque;
    private bool upButton = true;

    void Start() { }

    void Update()
    {
        Rotacion();
        if (Input.GetButtonUp("Fire2"))
            upButton = true;
        if (Input.GetButton("Fire2"))
        {
            // Suelta la última pieza unida: los hijos del primer hijo del último hijo quedan
            // «libres» y el primero de ellos se desconecta (su abuelo se queda sin padre).
            if (transform.childCount > 4 && upButton)
            {
                for (int i = 0; i < transform.GetChild(transform.childCount - 1).GetChild(0).childCount; i++)
                    transform.GetChild(transform.childCount - 1).GetChild(0).GetChild(i).GetComponent<Union>().estado = "libre";
                transform.GetChild(transform.childCount - 1).GetChild(0).GetChild(0).GetComponent<Union>().desconectar();
            }
            upButton = false;
        }
    }

    void Rotacion()
    {
        transform.Rotate(0f, 0f, -(Input.GetAxis("Rotacion") * velocidadRotacion * Time.deltaTime));
    }
}
