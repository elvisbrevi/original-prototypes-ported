using UnityEngine;

public class PlatformerCharacter2D : MonoBehaviour
{
	private bool facingRight = true;

	public float maxSpeed = 10f;

	[SerializeField]
	private float jumpForce = 400f;

	[SerializeField]
	[Range(0f, 1f)]
	private float crouchSpeed = 0.36f;

	[SerializeField]
	private bool airControl;

	[SerializeField]
	private LayerMask whatIsGround;

	private Transform groundCheck;

	private float groundedRadius = 0.2f;

	private bool grounded;

	private Transform ceilingCheck;

	private float ceilingRadius = 0.01f;

	public Animator anim;

	private bool doubleJump;

	public bool canDoubleJump;

	private AudioSource[] auds;

	private bool pasos = true;

	public GameObject polvo;

	public GameObject hitSalto;

	private void Awake()
	{
		groundCheck = base.transform.Find("GroundCheck");
		ceilingCheck = base.transform.Find("CeilingCheck");
		anim = GetComponent<Animator>();
		auds = GetComponents<AudioSource>();
	}

	private void FixedUpdate()
	{
		grounded = Physics2D.OverlapCircle(groundCheck.position, groundedRadius, whatIsGround);
		if (grounded & !anim.GetBool("Dead") & (GameObject.FindGameObjectsWithTag("Particula").Length == 0))
		{
			GameObject gameObject = (GameObject)Object.Instantiate((Object)polvo);
			gameObject.transform.position = new Vector3(groundCheck.position.x - 1f, groundCheck.position.y, groundCheck.position.z);
			gameObject.renderer.sortingLayerName = "Player";
			gameObject.transform.parent = base.transform;
		}
		if (!grounded && Time.timeScale > 0f)
		{
			if (HUD.estadoStatic.ToString() == "ShareGUI")
			{
				auds[1].Stop();
			}
			else
			{
				auds[1].Play();
			}
		}
		else if (HUD.estadoStatic.ToString() == "ShareGUI")
		{
			auds[1].Stop();
		}
		anim.SetBool("Ground", grounded);
		anim.SetFloat("vSpeed", ((Component)this).rigidbody2D.velocity.y);
		if (grounded && canDoubleJump)
		{
			doubleJump = false;
		}
		if (!grounded & (GameObject.FindGameObjectsWithTag("Particula").Length > 0))
		{
			GameObject.FindGameObjectWithTag("Particula").transform.parent = null;
		}
	}

	public void Move(float move, bool crouch, bool jump)
	{
		if (grounded || airControl)
		{
			move = ((!crouch) ? move : (move * crouchSpeed));
			anim.SetFloat("Speed", Mathf.Abs(move));
			((Component)this).rigidbody2D.velocity = new Vector2(move * maxSpeed, ((Component)this).rigidbody2D.velocity.y);
			if (move > 0f && !facingRight)
			{
				Flip();
			}
			else if (move < 0f && facingRight)
			{
				Flip();
			}
		}
		if ((grounded || (!doubleJump & canDoubleJump)) && jump)
		{
			anim.SetBool("Ground", value: false);
			((Component)this).rigidbody2D.velocity = new Vector2(((Component)this).rigidbody2D.velocity.x, 0f);
			if (maxSpeed > 0f)
			{
				((Component)this).rigidbody2D.AddForce(new Vector2(0f, jumpForce));
				auds[0].Play();
				GameObject gameObject = (GameObject)Object.Instantiate((Object)hitSalto);
				gameObject.transform.position = new Vector3(groundCheck.position.x + 1f, groundCheck.position.y + 1f, groundCheck.position.z);
				gameObject.renderer.sortingLayerName = "Player";
			}
			if (!grounded & canDoubleJump)
			{
				doubleJump = true;
			}
		}
	}

	private void Flip()
	{
		facingRight = !facingRight;
		Vector3 localScale = base.transform.localScale;
		localScale.x *= -1f;
		base.transform.localScale = localScale;
	}
}
