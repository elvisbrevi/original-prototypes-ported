using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Moneda
{
	public Texture2D textura;

	public Vector2 posicion;

	public Vector2 posicionInicial;

	public Rectangle cuerpo;

	public float zoom;

	public bool colisionIzquierda;

	public bool colisionDerecha;

	public bool colisionSuperior;

	public bool activa;

	public Vector2 direccion;

	public void cargar(ContentManager content)
	{
		textura = content.Load<Texture2D>("Graficos/Items/moneda");
		zoom = 0.2f;
		direccion = new Vector2(5f, 0f);
	}

	public void actualizar()
	{
		cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, (int)((float)textura.Width * zoom), (int)((float)textura.Height * zoom));
	}

	public void dibujar(SpriteBatch sprite)
	{
		sprite.Draw(textura, posicion, new Rectangle(0, 0, textura.Width, textura.Height), Color.Gold, 0f, Vector2.Zero, zoom, SpriteEffects.None, 1f);
	}

	public void mover()
	{
		if (colisionDerecha)
		{
			direccion = new Vector2(-3f, 0f);
		}
		if (colisionIzquierda)
		{
			direccion = new Vector2(3f, 0f);
		}
		posicion.X += direccion.X;
	}

	private void salta()
	{
	}
}
