using UnityEngine;

public class TimControllerScript : MonoBehaviour
{
	public float maxSpeed = 10f;

	private bool facingRight = true;

	private Animator anim;

	private bool grounded = true;

	public Transform groundCheck;

	private float groundRadius = 0.3f;

	public LayerMask whatIsGround;

	public float jumpForce;

	public bool canDoubleJump;

	private bool doubleJump;

	public bool controlJump = true;

	private void Start()
	{
		anim = GetComponent<Animator>();
	}

	private void FixedUpdate()
	{
		grounded = Physics2D.OverlapCircle(groundCheck.position, groundRadius, whatIsGround);
		anim.SetBool("Ground", grounded);
		if (grounded)
		{
			doubleJump = false;
		}
		anim.SetFloat("vSpeed", ((Component)this).rigidbody2D.velocity.y);
		if (!(!controlJump & !grounded))
		{
			float axis = Input.GetAxis("Horizontal");
			anim.SetFloat("Speed", Mathf.Abs(axis));
			((Component)this).rigidbody2D.velocity = new Vector2(axis * maxSpeed, ((Component)this).rigidbody2D.velocity.y);
			if (axis > 0f && !facingRight)
			{
				Flip();
			}
			else if (axis < 0f && facingRight)
			{
				Flip();
			}
		}
	}

	private void Update()
	{
		if ((grounded || (!doubleJump && canDoubleJump)) && Input.GetKeyDown(KeyCode.Space))
		{
			anim.SetBool("Ground", value: false);
			((Component)this).rigidbody2D.AddForce(new Vector2(0f, jumpForce));
			if (canDoubleJump && !doubleJump && !grounded)
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
