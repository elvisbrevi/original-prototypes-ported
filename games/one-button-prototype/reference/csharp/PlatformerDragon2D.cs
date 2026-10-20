using UnityEngine;

namespace UnityStandardAssets._2D;

public class PlatformerDragon2D : MonoBehaviour
{
	private const float k_GroundedRadius = 0.2f;

	private const float k_CeilingRadius = 0.01f;

	[SerializeField]
	private float m_MaxSpeed = 10f;

	[SerializeField]
	[Range(0f, 1f)]
	private float m_CrouchSpeed = 0.36f;

	[SerializeField]
	private LayerMask m_WhatIsGround;

	private Transform m_GroundCheck;

	public static bool m_Grounded;

	private Transform m_CeilingCheck;

	public Rigidbody2D m_Rigidbody2D;

	private bool m_FacingRight;

	private void Awake()
	{
		m_Rigidbody2D = GetComponent<Rigidbody2D>();
	}

	private void FixedUpdate()
	{
		m_Grounded = false;
	}

	public void Move(float move, bool crouch, bool jump)
	{
		move = ((!crouch) ? move : (move * m_CrouchSpeed));
		m_Rigidbody2D.velocity = new Vector2(move * m_MaxSpeed, m_Rigidbody2D.velocity.y);
		if (move > 0f && !m_FacingRight)
		{
			Flip();
		}
		else if (move < 0f && m_FacingRight)
		{
			Flip();
		}
	}

	private void Flip()
	{
		m_FacingRight = !m_FacingRight;
		Vector3 localScale = base.transform.localScale;
		localScale.x *= -1f;
		base.transform.localScale = localScale;
	}
}
