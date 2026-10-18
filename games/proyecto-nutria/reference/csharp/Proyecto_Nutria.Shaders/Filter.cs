using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria.Shaders;

internal class Filter : Post_Processing
{
	public override void cargar(ContentManager content)
	{
		efecto = content.Load<Effect>("Shaders/filter");
	}

	public override void actualizar()
	{
	}

	public override void inicio(SpriteBatch sprite, Camara camara)
	{
		if (activo)
		{
			efecto.Parameters["burn"].SetValue(0.15f);
			efecto.Parameters["saturation"].SetValue(0.05f);
			efecto.Parameters["r"].SetValue(1f);
			efecto.Parameters["g"].SetValue(0.98f);
			efecto.Parameters["b"].SetValue(0.85f);
			efecto.Parameters["brite"].SetValue(0.05f);
			efecto.Begin();
			sprite.End();
			sprite.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, camara.Camara_.transform);
			EffectPass effectPass = efecto.CurrentTechnique.Passes[0];
			effectPass.Begin();
		}
	}

	public override void fin(SpriteBatch sprite)
	{
		if (activo)
		{
			EffectPass effectPass = efecto.CurrentTechnique.Passes[0];
			effectPass.End();
			efecto.End();
		}
	}
}
