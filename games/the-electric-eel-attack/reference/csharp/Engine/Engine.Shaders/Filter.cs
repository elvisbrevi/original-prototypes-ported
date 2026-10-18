using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Shaders;

public class Filter : Post_Processing
{
	public override void cargar(ContentManager content)
	{
		efecto = content.Load<Effect>("Shaders/filter");
	}

	public override void actualizar(GameTime tiempo)
	{
	}

	public override void iniciar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		if (Activo)
		{
			efecto.Begin();
			efecto.Parameters["burn"].SetValue(burn);
			efecto.Parameters["saturation"].SetValue(saturation);
			efecto.Parameters["r"].SetValue(r);
			efecto.Parameters["g"].SetValue(g);
			efecto.Parameters["b"].SetValue(b);
			efecto.Parameters["brite"].SetValue(brite);
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
