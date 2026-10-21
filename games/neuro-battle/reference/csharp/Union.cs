// Reconstruido del asm.js (reference/il2cpp/asmjs/Union.js); en cada «pata» de las neuronas.
using UnityEngine;

public class Union : MonoBehaviour
{
    // Solo lo escribe BaseRotacion («libre»); nadie lo lee.
    public string estado = "libre";
    public AudioClip shootSound;
    private AudioSource source;

    void Awake()
    {
        source = GetComponent<AudioSource>();
    }

    void Start() { }

    void Update()
    {
        BaseRotacion.choque = false;
    }

    // La pata cuelga de la neurona por dos niveles (neurona → centro → pata): desconectar deja a
    // la neurona sin padre.
    public void desconectar()
    {
        transform.parent.parent.parent = null;
    }

    void OnTriggerEnter2D(Collider2D other)
    {
        // «&» sin cortocircuito: el nombre del abuelo se lee aunque el otro no sea una pata.
        if (other.name.Contains("pata") & transform.parent.parent.name.Contains("ataque"))
            transform.parent.parent.GetComponent<disparar>().valido = true;
        BaseRotacion.choque = true;
        // Una neurona suelta que toca una pata se une a la neurona de esa pata (al padre de la
        // pata, su «centro»), conservando su posición en el mundo.
        if (other.name.Contains("pata") && transform.parent.parent.parent == null)
        {
            transform.parent.parent.SetParent(other.transform.parent);
            source.PlayOneShot(shootSound, 100f);
        }
        BaseRotacion.choque = true;
    }
}
