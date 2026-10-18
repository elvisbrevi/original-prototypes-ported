using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Fisica
{
	public float[] gravedadPersonaje = new float[2];

	public float[] gravedadMonedas = new float[100];

	private Curve curva = new Curve();

	public void actualizar(Nivel nivel, GameTime tiempo)
	{
		gravedad(nivel.personajes, nivel.monedas);
		colisionPlataforma(nivel.plataformas, nivel.personajes, nivel.monedas);
		colisionCajas(nivel.cajas, nivel.personajes);
		colisionBalas(nivel.personajes, nivel, tiempo);
		colisionMonedas(nivel.monedas, nivel.personajes, nivel);
	}

	private void gravedad(List<Personaje> personajes, List<Moneda> monedas)
	{
		for (int i = 0; i < personajes.Count; i++)
		{
			if (personajes[i].Estado_ == Personaje.Estado.Saltando)
			{
				gravedadPersonaje[i] = 0f;
			}
			if (personajes[i].Estado_ != Personaje.Estado.Saltando)
			{
				personajes[i].Posicion.Y += gravedadPersonaje[i];
				if (gravedadPersonaje[i] <= 8f)
				{
					gravedadPersonaje[i] += 1f;
				}
			}
		}
		for (int i = 0; i < monedas.Count; i++)
		{
			if (monedas[i].colisionSuperior)
			{
				gravedadMonedas[i] = 0f;
				continue;
			}
			monedas[i].posicion.Y += gravedadMonedas[i];
			if (gravedadMonedas[i] <= 8f)
			{
				gravedadMonedas[i] += 1f;
			}
		}
	}

	private void colisionMonedas(List<Moneda> monedas, List<Personaje> personajes, Nivel nivel)
	{
		foreach (Personaje personaje in personajes)
		{
			foreach (Moneda moneda in monedas)
			{
				if (moneda.cuerpo.Intersects(personaje.Cuerpo))
				{
					personaje.monedas++;
					moneda.posicion = moneda.posicionInicial;
					moneda.activa = false;
				}
				if (!nivel.limites.Contains(moneda.cuerpo))
				{
					moneda.posicion = moneda.posicionInicial;
					moneda.activa = false;
				}
			}
		}
	}

	private void colisionBalas(List<Personaje> personajes, Nivel nivel, GameTime tiempo)
	{
		for (int i = 0; i < personajes.Count; i++)
		{
			if (personajes[i].golpeado)
			{
				if ((float)personajes[i].posicionGolpe < personajes[i].Posicion.Y)
				{
					personajes[i].Posicion.Y -= (float)tiempo.ElapsedGameTime.TotalMilliseconds * 1.5f;
					personajes[i].rotacion += 0.5f;
					personajes[i].color = Color.Red;
				}
				else
				{
					personajes[i].rotacion = 0f;
					personajes[i].golpeado = false;
				}
				personajes[i].duracionGolpe -= tiempo.ElapsedGameTime.TotalSeconds * 1.0;
			}
			else
			{
				personajes[i].posicionGolpe = 0;
				personajes[i].duracionGolpe = 2.0;
				personajes[i].color = Color.White;
			}
			if (personajes[i].vida <= 0f)
			{
				personajes[i].vida = 200f;
				personajes[i].monedas = personajes[i].monedas - 50;
			}
			if (i == 1 && personajes[0].arma != null)
			{
				foreach (Bala bala in personajes[0].arma.balas)
				{
					if (personajes[i].Cuerpo.Intersects(bala.cuerpo) && bala.activa)
					{
						personajes[i].vida -= 1f;
						personajes[i].posicionGolpe = (int)(personajes[i].Posicion.Y - 250f);
						personajes[i].golpeado = true;
					}
				}
			}
			else
			{
				if (i != 0 || personajes[1].arma == null)
				{
					continue;
				}
				foreach (Bala bala2 in personajes[1].arma.balas)
				{
					if (personajes[i].Cuerpo.Intersects(bala2.cuerpo) && bala2.activa)
					{
						personajes[i].vida -= 1f;
						personajes[i].posicionGolpe = (int)(personajes[i].Posicion.Y - 250f);
						personajes[i].golpeado = true;
					}
				}
			}
		}
	}

	private void colisionCajas(List<Caja> cajas, List<Personaje> personajes)
	{
		foreach (Personaje personaje in personajes)
		{
			foreach (Caja caja in cajas)
			{
				if (personaje.Cuerpo.Intersects(caja.cuerpo) && caja.activa)
				{
					caja.activa = false;
					personaje.arma = caja.arma;
					personaje.armado = true;
					if (personaje.arma.nombre.Equals("Lanzallamas"))
					{
						personaje.cantidadBalas = 10;
					}
					else if (personaje.arma.nombre.Equals("Laser"))
					{
						personaje.cantidadBalas = 10;
					}
					caja.arma = null;
				}
			}
		}
	}

	private void colisionPlataforma(List<Plataforma> plataformas, List<Personaje> personajes, List<Moneda> monedas)
	{
		for (int i = 0; i < personajes.Count; i++)
		{
			personajes[i].Cuerpo = new Rectangle((int)personajes[i].Posicion.X, (int)personajes[i].Posicion.Y, (int)((float)personajes[i].Cuadro.Width * personajes[i].Zoom), (int)((float)personajes[i].Cuadro.Height * personajes[i].Zoom));
			personajes[i].origen = new Vector2(personajes[i].Cuadro.Width / 2, personajes[i].Cuadro.Height / 2);
			personajes[i].ColisionSuperior = false;
			personajes[i].ColisionInferior = false;
			personajes[i].ColisionDerecha = false;
			personajes[i].ColisionIzquierda = false;
			foreach (Plataforma plataforma in plataformas)
			{
				if (!personajes[i].Cuerpo.Intersects(plataforma.Cuerpo))
				{
					continue;
				}
				if (plataforma.Colision_ == Plataforma.Colision.Lateral || plataforma.Colision_ == Plataforma.Colision.Total)
				{
					if (personajes[i].Posicion.X >= (float)plataforma.Cuerpo.X + ((float)plataforma.Cuerpo.Width - (gravedadPersonaje[i] + personajes[i].VelocidadMovimiento)) - 10f && personajes[i].Posicion.Y + (float)personajes[i].Cuerpo.Height > (float)plataforma.Cuerpo.Y + (gravedadPersonaje[i] + personajes[i].VelocidadMovimiento))
					{
						personajes[i].ColisionIzquierda = true;
						personajes[i].VelocidadMovimiento = 0f;
						personajes[i].Posicion.X = (int)MathHelper.Clamp(personajes[i].Posicion.X, plataforma.Cuerpo.Right, personajes[i].Cuerpo.Right);
					}
					if (personajes[i].Posicion.X + (float)personajes[i].Cuerpo.Width <= (float)plataforma.Cuerpo.X + (gravedadPersonaje[i] + personajes[i].VelocidadMovimiento) + 10f && personajes[i].Posicion.Y + (float)personajes[i].Cuerpo.Height > (float)plataforma.Cuerpo.Y + (gravedadPersonaje[i] + personajes[i].VelocidadMovimiento) + 10f)
					{
						personajes[i].ColisionDerecha = true;
						personajes[i].VelocidadMovimiento = 0f;
						personajes[i].Posicion.X = (int)MathHelper.Clamp(personajes[i].Posicion.X, 0f, plataforma.Cuerpo.Left - personajes[i].Cuerpo.Width);
					}
				}
				if ((plataforma.Colision_ == Plataforma.Colision.Superior || plataforma.Colision_ == Plataforma.Colision.Total) && personajes[i].Posicion.Y + (float)personajes[i].Cuerpo.Height <= (float)plataforma.Cuerpo.Y + (gravedadPersonaje[i] + 1f) && personajes[i].Posicion.X + (float)personajes[i].Cuerpo.Width > (float)plataforma.Cuerpo.X)
				{
					personajes[i].Posicion.Y = plataforma.Cuerpo.Y - personajes[i].Cuerpo.Height;
					personajes[i].ColisionSuperior = true;
					if (plataforma.Ida)
					{
						if (plataforma.Movimiento_ == Plataforma.Movimiento.Vertical)
						{
							personajes[i].Posicion.Y += plataforma.velocidadMovimiento;
						}
						else if (plataforma.Movimiento_ == Plataforma.Movimiento.Horizontal)
						{
							personajes[i].Posicion.X += plataforma.velocidadMovimiento;
						}
					}
					else if (!plataforma.Ida)
					{
						if (plataforma.Movimiento_ == Plataforma.Movimiento.Vertical)
						{
							personajes[i].Posicion.Y -= plataforma.velocidadMovimiento;
						}
						else if (plataforma.Movimiento_ == Plataforma.Movimiento.Horizontal)
						{
							personajes[i].Posicion.X -= plataforma.velocidadMovimiento;
						}
					}
				}
				if ((plataforma.Colision_ == Plataforma.Colision.Inferior || plataforma.Colision_ == Plataforma.Colision.Total) && personajes[i].Posicion.Y >= (float)plataforma.Cuerpo.Y + ((float)plataforma.Cuerpo.Height - (gravedadPersonaje[i] + personajes[i].VelocidadMovimiento)) && personajes[i].Posicion.X + (float)personajes[i].Cuerpo.Width > (float)plataforma.Cuerpo.X)
				{
					personajes[i].Posicion.Y = (int)MathHelper.Clamp(personajes[i].Posicion.Y, plataforma.Cuerpo.Bottom + 10, 0f);
					personajes[i].ColisionInferior = true;
				}
			}
		}
		for (int i = 0; i < monedas.Count; i++)
		{
			monedas[i].colisionSuperior = false;
			monedas[i].colisionDerecha = false;
			monedas[i].colisionIzquierda = false;
			foreach (Plataforma plataforma2 in plataformas)
			{
				if (!monedas[i].cuerpo.Intersects(plataforma2.Cuerpo))
				{
					continue;
				}
				if (plataforma2.Colision_ == Plataforma.Colision.Lateral || plataforma2.Colision_ == Plataforma.Colision.Total)
				{
					if (monedas[i].posicion.X >= (float)plataforma2.Cuerpo.X + ((float)plataforma2.Cuerpo.Width - (gravedadMonedas[i] + monedas[i].direccion.X)) - 10f && monedas[i].posicion.Y + (float)monedas[i].cuerpo.Height > (float)plataforma2.Cuerpo.Y + (gravedadMonedas[i] + monedas[i].direccion.X))
					{
						monedas[i].colisionIzquierda = true;
						monedas[i].direccion.X = 0f;
						monedas[i].posicion.X = (int)MathHelper.Clamp(monedas[i].posicion.X, plataforma2.Cuerpo.Right, monedas[i].cuerpo.Right);
					}
					if (monedas[i].posicion.X + (float)monedas[i].cuerpo.Width <= (float)plataforma2.Cuerpo.X + (gravedadMonedas[i] + monedas[i].direccion.X) + 10f && monedas[i].posicion.Y + (float)monedas[i].cuerpo.Height > (float)plataforma2.Cuerpo.Y + (gravedadMonedas[i] + monedas[i].direccion.X) + 10f)
					{
						monedas[i].colisionDerecha = true;
						monedas[i].direccion.X = 0f;
						monedas[i].posicion.X = (int)MathHelper.Clamp(monedas[i].posicion.X, 0f, plataforma2.Cuerpo.Left - monedas[i].cuerpo.Width);
					}
				}
				if ((plataforma2.Colision_ != Plataforma.Colision.Superior && plataforma2.Colision_ != Plataforma.Colision.Total) || !(monedas[i].posicion.Y + (float)monedas[i].cuerpo.Height <= (float)plataforma2.Cuerpo.Y + (gravedadMonedas[i] + 1f)) || !(monedas[i].posicion.X + (float)monedas[i].cuerpo.Width > (float)plataforma2.Cuerpo.X))
				{
					continue;
				}
				monedas[i].posicion.Y = plataforma2.Cuerpo.Y - monedas[i].cuerpo.Height;
				monedas[i].colisionSuperior = true;
				if (plataforma2.Ida)
				{
					if (plataforma2.Movimiento_ == Plataforma.Movimiento.Vertical)
					{
						monedas[i].posicion.Y += monedas[i].direccion.X;
					}
					else if (plataforma2.Movimiento_ == Plataforma.Movimiento.Horizontal)
					{
						monedas[i].posicion.X += monedas[i].direccion.X;
					}
				}
				else if (!plataforma2.Ida)
				{
					if (plataforma2.Movimiento_ == Plataforma.Movimiento.Vertical)
					{
						monedas[i].posicion.Y -= monedas[i].direccion.X;
					}
					else if (plataforma2.Movimiento_ == Plataforma.Movimiento.Horizontal)
					{
						monedas[i].posicion.X -= monedas[i].direccion.X;
					}
				}
			}
		}
	}
}
