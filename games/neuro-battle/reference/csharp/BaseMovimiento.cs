// Reconstruido del asm.js (reference/il2cpp/asmjs/BaseMovimiento.js); en neurona_base.
using UnityEngine;

public class BaseMovimiento : MonoBehaviour
{
    public float velocidadMovimiento = 1f;

    void Start() { }

    void Update()
    {
        Movimiento();
    }

    void Movimiento()
    {
        // Translate en el espacio propio: la neurona_base no gira (gira su hijo «centro»).
        transform.Translate(new Vector3(Input.GetAxis("Horizontal"), Input.GetAxis("Vertical"), 0f)
                            * velocidadMovimiento * Time.deltaTime);
    }
}
