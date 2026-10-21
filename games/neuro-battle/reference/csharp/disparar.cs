// Reconstruido del asm.js (reference/il2cpp/asmjs/disparar.js); en el prefab «neurona_ataque».
using UnityEngine;

public class disparar : MonoBehaviour
{
    // El inicializador crea un GameObject vacío al construir el script (Unity lo impide en un
    // constructor y escribe un error); el valor serializado (el prefab «bala») lo reemplaza.
    public GameObject bala = new GameObject();
    public Vector3 bulletOffset = new Vector3(0f, 0.5f, 0f);
    private bool disparo;
    // Lo pone Union cuando la neurona de ataque se une a otra: solo dispara si está unida.
    public bool valido;
    public AudioClip shootSound;
    private AudioSource source;

    void Awake()
    {
        source = GetComponent<AudioSource>();
    }

    void Start() { }

    void Update()
    {
        Vector3 desplazamiento = Vector3.zero;
        if (valido)
        {
            if (Input.GetKeyUp(KeyCode.RightShift))
                disparo = false;
            if (Input.GetKey(KeyCode.RightShift) && !disparo)
            {
                desplazamiento = transform.rotation * bulletOffset;
                GameObject b = (GameObject)Object.Instantiate(bala, transform.position + desplazamiento, transform.rotation);
                source.PlayOneShot(shootSound, 100f);
                disparo = true;
            }
        }
    }
}
