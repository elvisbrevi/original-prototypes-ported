using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Microsoft.Xna.Framework.Input;

namespace Proyecto_Nutria;

public class Camera2D : GameComponent
{
	public Vector2 position;

	public float rotation;

	public float scale;

	public float speed;

	public Matrix transform;

	public GraphicsDevice graphicsDevice;

	public bool manualcamera;

	public Camera2D(Game game, bool manualcamera)
		: base(game)
	{
		graphicsDevice = game.GraphicsDevice;
		this.manualcamera = manualcamera;
	}

	public override void Initialize()
	{
		position = new Vector2(0f, 0f);
		rotation = 0f;
		scale = 1f;
		speed = 10f;
		manualcamera = false;
		base.Initialize();
	}

	public override void Update(GameTime gameTime)
	{
		if (manualcamera)
		{
			if (Keyboard.GetState().IsKeyDown(Keys.D) || GamePad.GetState(PlayerIndex.One).DPad.Left == ButtonState.Pressed || GamePad.GetState(PlayerIndex.One).ThumbSticks.Left.X < 0f)
			{
				position.X += speed;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.A) || GamePad.GetState(PlayerIndex.One).DPad.Right == ButtonState.Pressed || GamePad.GetState(PlayerIndex.One).ThumbSticks.Left.X > 0f)
			{
				position.X -= speed;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.W) || GamePad.GetState(PlayerIndex.One).DPad.Down == ButtonState.Pressed || GamePad.GetState(PlayerIndex.One).ThumbSticks.Left.Y < 0f)
			{
				position.Y -= speed;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.S) || GamePad.GetState(PlayerIndex.One).DPad.Up == ButtonState.Pressed || GamePad.GetState(PlayerIndex.One).ThumbSticks.Left.Y > 0f)
			{
				position.Y += speed;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.Q) || GamePad.GetState(PlayerIndex.One).ThumbSticks.Right.X < 0f)
			{
				rotation += 0.01f;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.E) || GamePad.GetState(PlayerIndex.One).ThumbSticks.Right.X > 0f)
			{
				rotation -= 0.01f;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.C) || GamePad.GetState(PlayerIndex.One).Triggers.Right > 0f)
			{
				scale += 0.01f;
			}
			if (Keyboard.GetState().IsKeyDown(Keys.Z) || GamePad.GetState(PlayerIndex.One).Triggers.Left > 0f)
			{
				scale -= 0.01f;
			}
		}
		transform = Matrix.CreateScale(scale) * Matrix.CreateRotationZ(rotation) * Matrix.CreateTranslation(new Vector3((0f - position.X) * scale, (0f - position.Y) * scale, 0f));
		base.Update(gameTime);
	}
}
