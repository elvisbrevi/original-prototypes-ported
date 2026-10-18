using System;
using System.Collections.Generic;
using Iman.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Fisicas;

internal class Fisica
{
	private static Rectangle pColision;

	public void actualizar(List<Objeto> objetos, GameTime tiempo)
	{
		aplicarColision(objetos);
		aplicarGravedad(objetos, tiempo);
		aplicarMagnetismo(objetos);
	}

	private void aplicarColision(List<Objeto> cuerpos)
	{
		foreach (Objeto cuerpo in cuerpos)
		{
			cuerpo.colisionSuperior = false;
			cuerpo.colisionInferior = false;
			cuerpo.colisionDerecha = false;
			cuerpo.colisionIzquierda = false;
		}
		foreach (Objeto cuerpo2 in cuerpos)
		{
			foreach (Objeto cuerpo3 in cuerpos)
			{
				if (!cuerpo3.cuerpo.Intersects(cuerpo2.cuerpo) || cuerpo2.tipoObjeto != Objeto.TipoObjeto.Plataforma || cuerpo2 == cuerpo3)
				{
					continue;
				}
				if (cuerpo2.colision == Objeto.Colision.Lateral || cuerpo2.colision == Objeto.Colision.Total)
				{
					if (cuerpo3.posicion.X >= (float)cuerpo2.cuerpo.X + ((float)cuerpo2.cuerpo.Width - (cuerpo3.fuerzaGravedad + (float)cuerpo3.velocidad)) - 10f && cuerpo3.posicion.Y + (float)cuerpo3.cuerpo.Height > (float)cuerpo2.cuerpo.Y + (cuerpo3.fuerzaGravedad + (float)cuerpo3.velocidad))
					{
						cuerpo3.colisionIzquierda = true;
						cuerpo3.posicion.X = (int)MathHelper.Clamp(cuerpo3.posicion.X, cuerpo2.cuerpo.Right, cuerpo3.cuerpo.Right);
					}
					if (cuerpo3.posicion.X + (float)cuerpo3.cuerpo.Width <= (float)cuerpo2.cuerpo.X + (cuerpo3.fuerzaGravedad + (float)cuerpo3.velocidad) + 10f && cuerpo3.posicion.Y + (float)cuerpo3.cuerpo.Height > (float)cuerpo2.cuerpo.Y + (cuerpo3.fuerzaGravedad + (float)cuerpo3.velocidad))
					{
						cuerpo3.colisionDerecha = true;
						cuerpo3.posicion.X = (int)MathHelper.Clamp(cuerpo3.posicion.X, 0f, cuerpo2.cuerpo.Left - cuerpo3.cuerpo.Width);
					}
				}
				if ((cuerpo2.colision == Objeto.Colision.Superior || cuerpo2.colision == Objeto.Colision.Total) && cuerpo3.posicion.Y + (float)cuerpo3.cuerpo.Height <= (float)cuerpo2.cuerpo.Y + (cuerpo3.fuerzaGravedad + 1f) && cuerpo3.posicion.X + (float)cuerpo3.cuerpo.Width > (float)cuerpo2.cuerpo.X)
				{
					cuerpo3.posicion.Y = (int)MathHelper.Clamp(cuerpo3.posicion.Y, 0f, cuerpo2.cuerpo.Y - cuerpo3.cuerpo.Height - cuerpo3.velocidad);
					cuerpo3.colisionSuperior = true;
					if (cuerpo2.ida)
					{
						if (cuerpo2.movimiento == Objeto.Movimiento.Vertical)
						{
							cuerpo3.posicion.Y += cuerpo2.velocidad;
						}
						else if (cuerpo2.movimiento == Objeto.Movimiento.Horizontal)
						{
							cuerpo3.posicion.X += cuerpo2.velocidad;
						}
					}
					else if (!cuerpo2.ida)
					{
						if (cuerpo2.movimiento == Objeto.Movimiento.Vertical)
						{
							cuerpo3.posicion.Y -= cuerpo2.velocidad;
						}
						else if (cuerpo2.movimiento == Objeto.Movimiento.Horizontal)
						{
							cuerpo3.posicion.X -= cuerpo2.velocidad;
						}
					}
				}
				if ((cuerpo2.colision == Objeto.Colision.Inferior || cuerpo2.colision == Objeto.Colision.Total) && cuerpo3.posicion.Y >= (float)cuerpo2.cuerpo.Y + ((float)cuerpo2.cuerpo.Height - (cuerpo3.fuerzaGravedad + (float)cuerpo3.velocidad)) && cuerpo3.posicion.X + (float)cuerpo3.cuerpo.Width > (float)cuerpo2.cuerpo.X)
				{
					cuerpo3.posicion.Y = (int)MathHelper.Clamp(cuerpo3.posicion.Y, cuerpo2.cuerpo.Bottom + 10, 0f);
					cuerpo3.colisionInferior = true;
				}
			}
		}
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
					pColision = new Rectangle(j, i, 5, 5);
					return true;
				}
			}
		}
		return false;
	}

	private void aplicarGravedad(List<Objeto> objetos, GameTime tiempo)
	{
		foreach (Objeto objeto in objetos)
		{
			if (!objeto.gravedad)
			{
				continue;
			}
			if (objeto.estado == Objeto.Estado.Saltando)
			{
				objeto.fuerzaGravedad = 0f;
			}
			if (objeto.estado != Objeto.Estado.Saltando)
			{
				objeto.posicion.Y += objeto.fuerzaGravedad;
				if (objeto.fuerzaGravedad <= 8f)
				{
					objeto.fuerzaGravedad += 1f;
				}
			}
		}
	}

	private void aplicarMagnetismo(List<Objeto> objetos)
	{
		foreach (Objeto objeto in objetos)
		{
			if (objeto.tipoObjeto != Objeto.TipoObjeto.Plataforma || !objeto.cuerpo.Intersects(objetos[objetos.Count - 1].iman) || !objetos[objetos.Count - 1].usandoIman)
			{
				continue;
			}
			if (objeto.gravedad)
			{
				if (objeto.cuerpo.Right < objetos[objetos.Count - 1].cuerpo.Left && !objeto.colisionDerecha)
				{
					objeto.posicion.X += 5f;
				}
				else if (objeto.cuerpo.Left > objetos[objetos.Count - 1].cuerpo.Right && !objeto.colisionIzquierda)
				{
					objeto.posicion.X -= 5f;
				}
			}
			else if (objeto.movimiento != Objeto.Movimiento.Ninguno && objeto.cuerpo.Bottom + 10 < objetos[objetos.Count - 1].cuerpo.Top && !objetos[objetos.Count - 1].colisionInferior)
			{
				objetos[objetos.Count - 1].posicion.Y -= 5f;
				objetos[objetos.Count - 1].fuerzaGravedad = 0f;
			}
		}
	}
}
