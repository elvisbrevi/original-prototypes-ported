// Reconstruido del asm.js (reference/il2cpp/asmjs/bala.js); en el prefab «bala».
using UnityEngine;

public class bala : MonoBehaviour
{
    void Start() { }

    void Update()
    {
        // Una unidad por frame hacia su arriba: la velocidad depende de los fps.
        transform.Translate(Vector3.up);
    }

    void OnTriggerEnter2D(Collider2D other)
    {
        if (other.name.Contains("enemigo") || other.name.Contains("lateral") || other.name.Contains("esquina"))
        {
            // En un borde, GetComponent devuelve null: el IL2CPP de Unity 5.1.1 no comprueba
            // nulos, así que la resta cae en la memoria baja del asm.js y la bala igual se destruye.
            other.transform.GetComponent<muerte_enemigo>().vida -= 1f;
            Object.Destroy(gameObject);
        }
    }

    void OnCollisionEnter2D(Collision2D coll)
    {
        if (coll.gameObject.name.Contains("Enemy") || coll.gameObject.name.Contains("esquina"))
            Object.Destroy(gameObject);
    }
}
