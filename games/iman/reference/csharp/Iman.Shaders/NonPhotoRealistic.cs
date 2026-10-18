using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Shaders;

internal class NonPhotoRealistic : Post_Processing
{
	private bool enableEdgeDetect;

	private bool enableSketch;

	private float sketchJitterSpeed;

	private Random random = new Random();

	private Texture2D sketchTexture;

	private Vector2 sketchJitter;

	private TimeSpan timeToNextJitter;

	public override void cargar(ContentManager Content)
	{
		sketchJitterSpeed = 0.05f;
		enableEdgeDetect = true;
		enableSketch = true;
		efecto = Content.Load<Effect>("Shaders/PostprocessEffect");
		sketchTexture = Content.Load<Texture2D>("Shaders/SketchTexture");
	}

	public override void finalizar(SpriteBatch spriteBatch)
	{
		throw new NotImplementedException();
	}

	public override void iniciar(SpriteBatch spriteBatch)
	{
		throw new NotImplementedException();
	}

	public override void actualizar(GameTime gameTime)
	{
		timeToNextJitter -= gameTime.ElapsedGameTime;
		if (timeToNextJitter <= TimeSpan.Zero)
		{
			sketchJitter.X = (float)random.NextDouble();
			sketchJitter.Y = (float)random.NextDouble();
			timeToNextJitter += TimeSpan.FromSeconds(sketchJitterSpeed);
		}
	}

	public override void iniciar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		GraphicsDevice graphicsDevice = graphics.GraphicsDevice;
		if (enableEdgeDetect || enableSketch)
		{
			graphicsDevice.SetRenderTarget(0, null);
			EffectParameterCollection parameters = efecto.Parameters;
			parameters["SketchJitter"].SetValue(sketchJitter);
			parameters["SketchTexture"].SetValue(sketchTexture);
			string name = "ColorSketch";
			efecto.CurrentTechnique = efecto.Techniques[name];
			efecto.Begin();
			efecto.CurrentTechnique.Passes[0].Begin();
		}
	}

	public override void finalizar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		if (enableEdgeDetect)
		{
			efecto.CurrentTechnique.Passes[0].End();
			efecto.End();
		}
	}
}
