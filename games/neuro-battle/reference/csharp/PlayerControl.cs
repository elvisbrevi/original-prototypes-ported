// Reconstruido del asm.js (reference/il2cpp/asmjs/PlayerControl.js): un control de plataformas de
// tutorial que ninguna escena usa (no se porta).
using UnityEngine;

public class PlayerControl : MonoBehaviour
{
    public bool grounded;
    public bool interact;
    public Transform jumpCheck;
    public Transform interactCheck;
    public float speed = 0.6f;
    private RaycastHit2D interacted;
    private Animator anim;

    void Start()
    {
        anim = GetComponent<Animator>();
    }

    void Update()
    {
        Movement();
        RaycastStuff();
    }

    void RaycastStuff()
    {
        Debug.DrawLine(transform.position, jumpCheck.position, Color.magenta);
        Debug.DrawLine(transform.position, interactCheck.position, Color.magenta);
        grounded = Physics2D.Linecast(transform.position, jumpCheck.position, 1 << LayerMask.NameToLayer("Ground"));
        if (Physics2D.Linecast(transform.position, interactCheck.position, 1 << LayerMask.NameToLayer("Guard")))
        {
            interacted = Physics2D.Linecast(transform.position, interactCheck.position, 1 << LayerMask.NameToLayer("Guard"));
            interact = true;
        }
        else
            interact = false;
        Physics2D.IgnoreLayerCollision(8, 10);
    }

    void Movement()
    {
        anim.SetFloat("speed", Mathf.Abs(Input.GetAxisRaw("Horizontal")));
        if (Input.GetAxisRaw("Horizontal") > 0f)
        {
            transform.Translate(Vector3.right * speed * Time.deltaTime);
            transform.eulerAngles = new Vector2(0f, 0f);
        }
        if (Input.GetAxisRaw("Horizontal") < 0f)
        {
            transform.Translate(Vector3.right * speed * Time.deltaTime);
            transform.eulerAngles = new Vector2(0f, 180f);
        }
        if (Input.GetKeyDown(KeyCode.Space) && grounded)
        {
            anim.SetTrigger("Jump");
            GetComponent<Rigidbody2D>().AddForce(transform.up * 200f);
        }
    }
}
