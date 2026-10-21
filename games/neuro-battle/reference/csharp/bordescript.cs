// Reconstruido del asm.js (reference/il2cpp/asmjs/bordescript.js); en las cuatro «pata» del borde.
using UnityEngine;

public class bordescript : MonoBehaviour
{
    void Start() { }

    void Update() { }

    void OnTriggerEnter2D(Collider2D other)
    {
        if (other.name.Contains("enemigo") || other.name.Contains("bala"))
            Object.Destroy(other.gameObject);
    }

    void OnCollisionEnter2D(Collision2D coll)
    {
        if (coll.gameObject.name.Contains("enemigo") || coll.gameObject.name.Contains("bala"))
            Object.Destroy(coll.gameObject);
    }
}
