using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Graficos;

internal class Ramas
{
	public enum Diseño
	{
		_01,
		_02,
		_03
	}

	public SpriteEffects direccion;

	public Diseño diseño;

	private Texture2D ramas;

	public float rotacion;

	public Vector2 posicion;

	private Vector2 origen;

	private bool ida;

	public void cargar(ContentManager content)
	{
		ramas = content.Load<Texture2D>("Graficos/Ramas/" + diseño);
		origen = new Vector2(ramas.Width / 2, ramas.Height / 2);
	}

	public void actualizar(GameTime tiempo)
	{
		if (ida)
		{
			rotacion += 0.001f;
			if (rotacion >= 0.1f)
			{
				ida = false;
			}
		}
		else
		{
			rotacion -= 0.001f;
			if (rotacion <= -0.1f)
			{
				ida = true;
			}
		}
	}

	public void dibujar(SpriteBatch spriteBatch)
	{
		spriteBatch.Draw(ramas, posicion, new Rectangle(0, 0, ramas.Width, ramas.Height), Color.White, rotacion, origen, 1f, direccion, 1f);
	}
}
