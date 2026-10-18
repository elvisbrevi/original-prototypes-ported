using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal abstract class Generador
{
	public enum Estado
	{
		Activo,
		Inactivo
	}

	public Particula[] Particulas;

	public Estado Estado_;

	protected Random random;

	protected Vector2 posicion;

	public abstract void cargar(ContentManager content, int cantidad, string textura);

	public abstract void actualizar(Vector2 posicion, GameTime tiempo);

	public abstract void dibujar(SpriteBatch sprite);

	protected void moverParticula(int i, GameTime tiempo)
	{
		if (Estado_ == Estado.Inactivo)
		{
			Particulas[i].tiempo += (float)tiempo.TotalGameTime.TotalSeconds;
			Particulas[i].actualizar();
			Particulas[i].posicion += Particulas[i].direccion;
			Particulas[i].color.A -= (byte)random.Next(5, 5);
			Particulas[i].rotacion += 0.1f;
		}
		else
		{
			Particulas[i].tiempo += (float)tiempo.TotalGameTime.TotalSeconds;
			Particulas[i].actualizar();
			Particulas[i].posicion += Particulas[i].direccion;
			Particulas[i].color.A -= (byte)random.Next(3, 5);
			Particulas[i].zoom += 0.01f;
			Particulas[i].rotacion += 0.1f;
		}
	}

	protected void reiniciarParticula(int i)
	{
		if (Particulas[i].color.A <= 5)
		{
			Particulas[i].tiempo = 0f;
			Particulas[i].posicion = posicion;
			Particulas[i].color.A = 254;
			Particulas[i].zoom = 0.1f;
			if (Estado_ == Estado.Inactivo)
			{
				Particulas[i].activa = false;
			}
			else
			{
				Particulas[i].activa = true;
			}
		}
	}
}
