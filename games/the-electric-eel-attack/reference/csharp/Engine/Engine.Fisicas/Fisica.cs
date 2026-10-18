using System;
using System.Collections.Generic;
using Engine.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Fisicas;

public static class Fisica
{
	private static void aplicarColision(Objeto objeto, List<Objeto> objetos)
	{
		foreach (Objeto objeto2 in objetos)
		{
			if (!objeto.Cuerpo.Intersects(objeto2.Cuerpo) || objeto2 == objeto || !objeto.Gravedad || !objeto2.Activo)
			{
				continue;
			}
			if (objeto2.Colision_ == Objeto.Colision.Lateral || objeto2.Colision_ == Objeto.Colision.Total)
			{
				if (objeto.Cuerpo.Left <= objeto2.Cuerpo.Right && (float)objeto.Cuerpo.Left + (objeto.Velocidad + 1f) > (float)objeto2.Cuerpo.Right && objeto.Cuerpo.Bottom > objeto2.Cuerpo.Top && objeto.Cuerpo.Top < objeto2.Cuerpo.Bottom)
				{
					objeto.Posicion.X = MathHelper.Clamp(objeto.Posicion.X, (float)objeto2.Cuerpo.Right - objeto.Velocidad, (float)objeto.Cuerpo.Right + objeto.Velocidad);
					objeto.colisionIzquierda = true;
					objeto2.colisionDerecha = true;
				}
				if (objeto.Cuerpo.Right >= objeto2.Cuerpo.Left && (float)objeto.Cuerpo.Right - (objeto.Velocidad + 1f) < (float)objeto2.Cuerpo.Left && objeto.Cuerpo.Bottom > objeto2.Cuerpo.Top && objeto.Cuerpo.Top < objeto2.Cuerpo.Bottom)
				{
					objeto.Posicion.X = MathHelper.Clamp(objeto.Posicion.X, (float)objeto.Cuerpo.Left - objeto.Velocidad, (float)objeto2.Cuerpo.Left - ((float)objeto.Cuerpo.Width - objeto.Velocidad));
					objeto.colisionDerecha = true;
					objeto2.colisionIzquierda = true;
				}
			}
			if ((objeto2.Colision_ == Objeto.Colision.Superior || objeto2.Colision_ == Objeto.Colision.Total) && objeto.Cuerpo.Bottom >= objeto2.Cuerpo.Top && objeto.Cuerpo.Top <= objeto2.Cuerpo.Top - (objeto.Cuerpo.Height - 9) && objeto.Cuerpo.Right - 5 > objeto2.Cuerpo.Left && objeto.Cuerpo.Left + 5 < objeto2.Cuerpo.Right)
			{
				objeto.Posicion.Y = MathHelper.Clamp(objeto.Posicion.Y, -100f, objeto2.Cuerpo.Top - (objeto.Cuerpo.Height - 1));
				objeto.colisionSuperior = true;
				objeto2.colisionInferior = true;
				if (objeto2.Movimento)
				{
					if (objeto2.idaY && objeto2.Destino.Y - objeto2.Inicio.Y != 0f)
					{
						objeto.Posicion.Y += objeto2.Velocidad;
					}
					else if (objeto2.Destino.Y - objeto2.Inicio.Y != 0f)
					{
						objeto.Posicion.Y -= objeto2.Velocidad;
					}
					if (objeto2.idaX && objeto2.Destino.X - objeto2.Inicio.X != 0f)
					{
						objeto.Posicion.X += objeto2.Velocidad;
					}
					else if (objeto2.Destino.X - objeto2.Inicio.X != 0f)
					{
						objeto.Posicion.X -= objeto2.Velocidad;
					}
				}
			}
			if ((objeto2.Colision_ == Objeto.Colision.Inferior || objeto2.Colision_ == Objeto.Colision.Total) && objeto.Cuerpo.Top <= objeto2.Cuerpo.Bottom && objeto.Cuerpo.Bottom >= objeto2.Cuerpo.Bottom + (objeto.Cuerpo.Height - 9) && objeto.Cuerpo.Right - 5 > objeto2.Cuerpo.Left && objeto.Cuerpo.Left + 5 < objeto2.Cuerpo.Right)
			{
				objeto.Posicion.Y = MathHelper.Clamp(objeto.Posicion.Y, objeto2.Cuerpo.Bottom - 1, objeto.Cuerpo.Bottom);
				objeto.colisionInferior = true;
				objeto2.colisionSuperior = true;
			}
		}
	}

	private static void aplicarGravedad(Objeto objeto, GameTime tiempo)
	{
		if (!objeto.Gravedad)
		{
			return;
		}
		if (objeto.colisionSuperior)
		{
			objeto.FuerzaGravedad = 0f;
			return;
		}
		objeto.Posicion.Y += objeto.FuerzaGravedad;
		if (objeto.FuerzaGravedad < 8f)
		{
			objeto.FuerzaGravedad += 0.5f;
		}
	}

	public static void aplicarFisicas(Objeto objeto, List<Objeto> objetos, GameTime tiempo)
	{
		aplicarGravedad(objeto, tiempo);
		aplicarColision(objeto, objetos);
	}

	private static bool IntersectPixels(Rectangle rectangleA, Color[] dataA, Rectangle rectangleB, Color[] dataB)
	{
		int num = Math.Max(rectangleA.Top, rectangleB.Top);
		int num2 = Math.Min(rectangleA.Bottom, rectangleB.Bottom);
		int num3 = Math.Max(rectangleA.Left, rectangleB.Left);
		int num4 = Math.Min(rectangleA.Right, rectangleB.Right);
		for (int i = num; i < num2; i++)
		{
			for (int j = num3; j < num4; j++)
			{
				Color color = dataA[j - rectangleA.Left + (i - rectangleA.Top) * rectangleA.Width];
				Color color2 = dataB[j - rectangleB.Left + (i - rectangleB.Top) * rectangleB.Width];
				if (color.A != 0 && color2.A != 0)
				{
					return true;
				}
			}
		}
		return false;
	}
}
