using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria.Shaders;

internal class Blur_Grayscale : Post_Processing
{
	public override void cargar(ContentManager content)
	{
		efecto = content.Load<Effect>("Shaders/blur_grayscale");
	}

	public override void actualizar()
	{
	}

	public override void inicio(SpriteBatch sprite, Camara camara)
	{
		if (activo)
		{
			efecto.Begin();
			sprite.End();
			sprite.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.SaveState, camara.Camara_.transform);
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
