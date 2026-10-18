using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

public class Plataforma
{
	public enum Movimiento
	{
		Ninguno,
		Vertical,
		Horizontal
	}

	public enum Diseño
	{
		Null,
		Pasto,
		Roca
	}

	public enum Colision
	{
		Superior,
		Lateral,
		Inferior,
		Total
	}

	private Texture2D textura;

	public Rectangle Cuerpo;

	public Vector2 Posicion;

	private Color color = Color.White;

	public Movimiento Movimiento_;

	public float posicionFinal;

	public float posicionInicial;

	public float Velocidad;

	public float zoom;

	public float velocidadMovimiento;

	public bool Ida;

	public bool Vuelta;

	public Diseño Diseño_;

	public Colision Colision_;

	public void cargar(ContentManager content)
	{
		Ida = true;
		Vuelta = false;
		textura = content.Load<Texture2D>("Graficos/Plataformas/" + Diseño_);
	}

	public void actualizar(GameTime tiempo)
	{
		if (Movimiento_ != Movimiento.Ninguno)
		{
			mover();
			Cuerpo = new Rectangle((int)Posicion.X, (int)Posicion.Y, textura.Width, textura.Height);
		}
	}

	public void dibujar(SpriteBatch sprite)
	{
		if (Diseño_ == Diseño.Null)
		{
			sprite.Draw(textura, Posicion, Cuerpo, Color.White);
		}
		else
		{
			sprite.Draw(textura, Cuerpo, Color.White);
		}
	}

	private void mover()
	{
		switch (Movimiento_)
		{
		case Movimiento.Horizontal:
			if (Ida)
			{
				Posicion.X += velocidadMovimiento;
				if (Posicion.X >= posicionFinal)
				{
					Ida = false;
					Vuelta = true;
				}
			}
			else if (Vuelta)
			{
				Posicion.X -= velocidadMovimiento;
				if (Posicion.X <= posicionInicial)
				{
					Ida = true;
					Vuelta = false;
				}
			}
			break;
		case Movimiento.Vertical:
			if (Ida)
			{
				Posicion.Y += velocidadMovimiento;
				if (Posicion.Y >= posicionFinal)
				{
					Ida = false;
					Vuelta = true;
				}
			}
			else if (Vuelta)
			{
				Posicion.Y -= velocidadMovimiento;
				if (Posicion.Y <= posicionInicial)
				{
					Ida = true;
					Vuelta = false;
				}
			}
			break;
		}
	}
}
