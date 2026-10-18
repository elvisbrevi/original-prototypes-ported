using Engine.Datos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Objetos;

public class Plataforma : Objeto
{
	public override void cargar(ContentManager content)
	{
		Textura = content.Load<Texture2D>("Graficos/" + NombreTextura);
	}

	public override void actualizar(GameTime tiempo)
	{
		Cuerpo = new Rectangle((int)Posicion.X, (int)Posicion.Y, Cuerpo.Width, Cuerpo.Height);
		if (Movimento)
		{
			mover();
		}
	}

	public override void dibujar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		if (fueraDeCamara)
		{
			spriteBatch.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None);
		}
		else
		{
			spriteBatch.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, Pantalla.getTransformationMatrix());
		}
		if (Escenario.MostrarCuerpos)
		{
			spriteBatch.Draw(Escenario.pixel, Cuerpo, new Color(Color.Blue, 100));
		}
		spriteBatch.End();
	}
}
