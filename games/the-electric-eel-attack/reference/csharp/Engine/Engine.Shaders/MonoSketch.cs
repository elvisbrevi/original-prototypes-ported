using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Shaders;

public class MonoSketch : Post_Processing
{
	private bool enableEdgeDetect;

	private bool enableSketch;

	private float sketchJitterSpeed;

	private Random random = new Random();

	private Texture2D sketchTexture;

	private Vector2 sketchJitter;

	private TimeSpan timeToNextJitter;

	private string effectTechniqueName = "MonoSketch";

	public override void cargar(ContentManager Content)
	{
		sketchJitterSpeed = 0.05f;
		enableEdgeDetect = true;
		enableSketch = true;
		efecto = Content.Load<Effect>("Shaders/NonPhotoRealistic");
		sketchTexture = Content.Load<Texture2D>("Shaders/SketchTexture");
	}

	public override void actualizar(GameTime gameTime)
	{
		if (Activo)
		{
			timeToNextJitter -= gameTime.ElapsedGameTime;
			if (timeToNextJitter <= TimeSpan.Zero)
			{
				sketchJitter.X = (float)random.NextDouble();
				sketchJitter.Y = (float)random.NextDouble();
				timeToNextJitter += TimeSpan.FromSeconds(sketchJitterSpeed);
			}
		}
	}

	public override void iniciar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		if (Activo)
		{
			GraphicsDevice graphicsDevice = graphics.GraphicsDevice;
			if (enableEdgeDetect || enableSketch)
			{
				graphicsDevice.SetRenderTarget(0, null);
				EffectParameterCollection parameters = efecto.Parameters;
				parameters["SketchJitter"].SetValue(sketchJitter);
				parameters["SketchTexture"].SetValue(sketchTexture);
				efecto.CurrentTechnique = efecto.Techniques[effectTechniqueName];
				efecto.Begin();
				efecto.CurrentTechnique.Passes[0].Begin();
			}
		}
	}

	public override void finalizar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		if (Activo && enableEdgeDetect)
		{
			efecto.CurrentTechnique.Passes[0].End();
			efecto.End();
		}
	}
}
