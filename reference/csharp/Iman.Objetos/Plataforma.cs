using Iman.Camaras;
using Iman.Controles;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Objetos;

internal class Plataforma : Objeto
{
	public override void cargar(ContentManager content)
	{
		tipoObjeto = TipoObjeto.Plataforma;
		ida = true;
		textura = content.Load<Texture2D>("Graficos/Plataformas/" + diseño);
		if (diseño != Diseño.NULL)
		{
			cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, textura.Width, textura.Height);
		}
	}

	public override void actualizar(GameTime tiempo, Control control)
	{
		mover();
		if (diseño != Diseño.NULL)
		{
			cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, textura.Width, textura.Height);
		}
	}

	public override void dibujar(SpriteBatch sprite, Camera2D camara)
	{
		if (diseño != Diseño.NULL)
		{
			sprite.Draw(textura, posicion, Color.White);
		}
		else
		{
			sprite.Draw(textura, posicion, new Rectangle(0, 0, cuerpo.Width, cuerpo.Height), new Color(Color.White, 180), 0f, Vector2.Zero, 1f, SpriteEffects.None, 1f);
		}
	}

	private void mover()
	{
		switch (movimiento)
		{
		case Movimiento.Horizontal:
			if (ida)
			{
				posicion.X += velocidad;
				if (posicion.X >= (float)posicionFinal)
				{
					ida = false;
					vuelta = true;
				}
			}
			else if (vuelta)
			{
				posicion.X -= velocidad;
				if (posicion.X <= (float)posicionInicial)
				{
					ida = true;
					vuelta = false;
				}
			}
			break;
		case Movimiento.Vertical:
			if (ida)
			{
				posicion.Y += velocidad;
				if (posicion.Y >= (float)posicionFinal)
				{
					ida = false;
					vuelta = true;
				}
			}
			else if (vuelta)
			{
				posicion.Y -= velocidad;
				if (posicion.Y <= (float)posicionInicial)
				{
					ida = true;
					vuelta = false;
				}
			}
			break;
		}
	}
}
