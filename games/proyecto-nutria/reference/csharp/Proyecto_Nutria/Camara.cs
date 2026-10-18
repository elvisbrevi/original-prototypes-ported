using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Camara
{
	public Camera2D Camara_;

	public Rectangle area;

	public bool activo;

	public float velocidadCamara;

	public void inicializar(Game game)
	{
		Camara_ = new Camera2D(game, manualcamera: true);
		game.Components.Add(Camara_);
		velocidadCamara = 6f;
	}

	public void actualizar(Nivel nivel)
	{
		float x = nivel.personajes[0].Posicion.X;
		float num = nivel.personajes[0].Posicion.X + (float)nivel.personajes[0].Cuerpo.Width;
		float y = nivel.personajes[0].Posicion.Y;
		float num2 = nivel.personajes[0].Posicion.Y + (float)nivel.personajes[0].Cuerpo.Height;
		if (activo)
		{
			if (x > nivel.personajes[1].Posicion.X)
			{
				x = nivel.personajes[1].Posicion.X;
			}
			if (num < nivel.personajes[1].Posicion.X + (float)nivel.personajes[1].Cuerpo.Width)
			{
				num = nivel.personajes[1].Posicion.X + (float)nivel.personajes[1].Cuerpo.Width;
			}
			if (y > nivel.personajes[1].Posicion.Y)
			{
				y = nivel.personajes[1].Posicion.Y;
			}
			if (num2 < nivel.personajes[1].Posicion.Y + (float)nivel.personajes[1].Cuerpo.Height)
			{
				num2 = nivel.personajes[1].Posicion.Y + (float)nivel.personajes[1].Cuerpo.Height;
			}
			area = new Rectangle((int)x, (int)y, (int)(num - x), (int)(num2 - y));
		}
		try
		{
			if (area.Width > 1000)
			{
				Camara_.scale = 1000f / (float)area.Width;
			}
		}
		catch (DivideByZeroException)
		{
		}
		if (Camara_.position.X < (float)area.X - 200f * Camara_.scale - 7f)
		{
			Camara_.position.X += velocidadCamara;
			if (velocidadCamara > 0f)
			{
				velocidadCamara -= 0.0001f;
			}
		}
		else if (Camara_.position.X > (float)area.X - 200f * Camara_.scale + 7f)
		{
			Camara_.position.X -= velocidadCamara;
			if (velocidadCamara < 6f)
			{
				velocidadCamara += 0.01f;
			}
		}
		Camara_.scale = MathHelper.Clamp(Camara_.scale, 0.38f, 0.6f);
		if (area.X > 1200)
		{
			Camara_.position.X = MathHelper.Clamp(Camara_.position.X, 0f, 1070f);
		}
		else
		{
			Camara_.position.X = MathHelper.Clamp(Camara_.position.X, 0f, nivel.limites.Right);
		}
		Camara_.position.Y = MathHelper.Clamp(Camara_.position.Y, 200f * Camara_.scale, 200f * Camara_.scale);
	}

	public void dibujar(SpriteBatch sprite)
	{
		sprite.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, Camara_.transform);
	}
}
