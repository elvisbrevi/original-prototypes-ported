// Reconstruido del asm.js (reference/il2cpp/asmjs/Aparicion.js); en la cámara del nivel.
using UnityEngine;

public class Aparicion : MonoBehaviour
{
    public float timerMax = 1f;
    private float timer;
    public GameObject[] neuronas;
    private bool zoom;
    // El tiempo de juego, que muerte_enemigo guarda como puntaje. No vuelve a 0 al cargar el
    // nivel otra vez: cuenta desde la primera vez que se entró al nivel.
    public static float contadorTiempo;

    void Start() { }

    void Update()
    {
        contadorTiempo += Time.deltaTime;
        if (Input.GetKey(KeyCode.Escape))
            Application.LoadLevel(0);
        if (Input.GetKeyUp(KeyCode.E))
            zoom = false;
        if (Input.GetKey(KeyCode.E) && !zoom)
        {
            if (Camera.main.orthographicSize == 9f)
                Camera.main.orthographicSize = 20f;
            else
                Camera.main.orthographicSize = 9f;
            zoom = true;
        }
        timer += Time.deltaTime;
        if (timer > timerMax)
        {
            try
            {
                // Random.Range con enteros: el máximo queda fuera.
                int i = Random.Range(0, 19);
                Vector3 posicion = new Vector3(Random.Range(-66, 19), Random.Range(-32, 18), 0f);
                Object.Instantiate(neuronas[i], posicion, Quaternion.identity);
            }
            catch (UnityException)
            {
            }
            timer = 0f;
        }
    }
}
