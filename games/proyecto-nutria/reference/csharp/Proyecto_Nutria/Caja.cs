using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Caja
{
	private Texture2D textura;

	private Color color = Color.White;

	public Rectangle cuerpo;

	public Vector2 posicion;

	public bool activa;

	public double tiempoActivacion;

	public Arma arma = new Pistola();

	private ContentManager content;

	public void cargar(ContentManager content)
	{
		this.content = content;
		textura = content.Load<Texture2D>("Graficos/Items/caja");
		activa = true;
		cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, textura.Width, textura.Height);
		arma.posicion = posicion;
		arma.cargar(content);
	}

	public void actualizar(GameTime tiempo)
	{
		aparicion(tiempo);
		if (arma == null)
		{
			arma = new Pistola();
			arma.posicion = posicion;
			arma.cargar(content);
		}
	}

	public void dibujar(SpriteBatch sprite)
	{
		sprite.Draw(textura, posicion, color);
	}

	private void aparicion(GameTime tiempo)
	{
		if (tiempoActivacion >= 5.0)
		{
			color.A++;
			if (color.A < 254)
			{
				color.A++;
			}
			else
			{
				activa = true;
				tiempoActivacion = 0.0;
			}
		}
		if (!activa && tiempoActivacion < 6.0)
		{
			color.A = 0;
			if (tiempoActivacion < 6.0)
			{
				tiempoActivacion += tiempo.ElapsedGameTime.TotalSeconds;
			}
		}
	}
}
