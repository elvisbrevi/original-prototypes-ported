// Reconstruido del asm.js (reference/il2cpp/asmjs/animacionUnion.js); en el «centro» de las neuronas.
using UnityEngine;

public class animacionUnion : MonoBehaviour
{
    private Animator anim;

    void Start()
    {
        anim = GetComponent<Animator>();
    }

    void Update()
    {
        // El asm.js solo asegura que BaseRotacion esté inicializada: lo que leyera de ella no
        // tenía efecto y el compilador lo quitó.
    }
}
