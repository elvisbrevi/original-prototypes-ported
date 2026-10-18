using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Shaders;

public class Blur_Grayscale : Post_Processing
{
	public override void cargar(ContentManager content)
	{
		efecto = content.Load<Effect>("Shaders/blur_grayscale");
	}

	public override void actualizar(GameTime tiempo)
	{
	}

	public override void iniciar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		if (Activo)
		{
			efecto.Begin();
			EffectPass effectPass = efecto.CurrentTechnique.Passes[0];
			effectPass.Begin();
		}
	}

	public override void finalizar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		if (Activo)
		{
			EffectPass effectPass = efecto.CurrentTechnique.Passes[0];
			effectPass.End();
			efecto.End();
		}
	}
}
