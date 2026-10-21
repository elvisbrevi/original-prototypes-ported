// Reconstruido del asm.js (reference/il2cpp/asmjs/muerte_enemigo.js); en el «enemigo».
using UnityEngine;

public class muerte_enemigo : MonoBehaviour
{
    public float vida = 5f;
    private Animator anim;

    void Start()
    {
        anim = GetComponent<Animator>();
    }

    void Update()
    {
        if (vida <= 0f)
            Object.Destroy(gameObject);
    }

    void OnTriggerEnter2D(Collider2D other)
    {
        // Choca con una neurona: mueren las dos; si era la del jugador, se guarda el tiempo y
        // termina la partida.
        if (other.name.Contains("neurona"))
        {
            if (other.name.Contains("neurona_base"))
            {
                PlayerPrefs.SetFloat("tiempo", Aparicion.contadorTiempo);
                Application.LoadLevel(3);
            }
            Object.Destroy(other.gameObject);
            Object.Destroy(gameObject);
        }
    }
}
