using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Explosion : Generador
{
	public override void cargar(ContentManager content, int cantidad, string textura)
	{
		Particulas = new Particula[cantidad];
		random = new Random();
		for (int i = 0; i < Particulas.Length; i++)
		{
			Particulas[i] = new Particula();
			Particulas[i].posicion = posicion;
			Particulas[i].color = Color.White;
			Particulas[i].zoom = 0.1f;
			Particulas[i].direccion = new Vector2(random.Next(-2, 2), random.Next(-2, 2));
			Particulas[i].cargar(content, "Particulas/" + textura);
		}
		Particulas[Particulas.Length - 1].tiempo = 5f;
	}

	public override void actualizar(Vector2 posicion, GameTime tiempo)
	{
		base.posicion = posicion;
		for (int i = 0; i < Particulas.Length; i++)
		{
			try
			{
				if (Particulas[i - 1].tiempo >= 3f)
				{
					moverParticula(i, tiempo);
				}
			}
			catch (IndexOutOfRangeException)
			{
				if (Particulas[Particulas.Length - 1].tiempo >= 3f)
				{
					moverParticula(i, tiempo);
				}
			}
			reiniciarParticula(i);
		}
	}

	public override void dibujar(SpriteBatch sprite)
	{
		for (int i = 0; i < Particulas.Length; i++)
		{
			Particulas[i].dibujar(sprite);
		}
	}
}
