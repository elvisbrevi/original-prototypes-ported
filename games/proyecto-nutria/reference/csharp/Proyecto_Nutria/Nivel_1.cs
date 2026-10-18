using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;
using Proyecto_Nutria.Iconos;
using Proyecto_Nutria.Shaders;

namespace Proyecto_Nutria;

internal class Nivel_1 : Nivel
{
	public override void cargar(ContentManager content, List<Control> controles)
	{
		marco = content.Load<Texture2D>("Graficos/Niveles/Nivel 1/Marco Coipo");
		cielo = content.Load<Texture2D>("Graficos/Niveles/Nivel 1/sky");
		scroll1 = content.Load<Texture2D>("Graficos/Niveles/Nivel 1/scroll1");
		scroll2 = content.Load<Texture2D>("Graficos/Niveles/Nivel 1/scroll2");
		diseñoTrasero = content.Load<Texture2D>("Graficos/Niveles/Nivel 1/plataformas");
		posicionDiseño = new Vector2(0f, 0f);
		colorDiseño = Color.White;
		limites = new Rectangle(50, 0, 2621, 1350);
		personajes = new List<Personaje>(2);
		Personaje personaje = new Coipo();
		personaje.Posicion = new Vector2(100f, 0f);
		personajes.Add(personaje);
		Personaje item = new Pollo();
		personaje.Posicion = new Vector2(200f, 0f);
		personajes.Add(item);
		cajas = new List<Caja>();
		crearCaja(new Vector2(1500f, 270f), content);
		crearCaja(new Vector2(500f, 270f), content);
		crearCaja(new Vector2(2500f, 270f), content);
		crearCaja(new Vector2(2700f, 490f), content);
		crearCaja(new Vector2(1100f, 490f), content);
		crearCaja(new Vector2(100f, 710f), content);
		crearCaja(new Vector2(1000f, 710f), content);
		crearCaja(new Vector2(2100f, 710f), content);
		crearCaja(new Vector2(2700f, 820f), content);
		plataformas = new List<Plataforma>();
		crearPlataforma(new Rectangle(-50, 0, 50, 1500), Plataforma.Colision.Lateral, content);
		crearPlataforma(new Rectangle(2850, 0, 50, 1500), Plataforma.Colision.Lateral, content);
		crearPlataforma(new Rectangle(30, 350, 620, 30), content);
		crearPlataforma(new Rectangle(900, 350, 1130, 30), content);
		crearPlataforma(new Rectangle(2230, 350, 650, 30), content);
		crearPlataforma(new Rectangle(580, 460, 430, 30), content);
		crearPlataformaMovediza(new Vector2(2030f, 460f), Plataforma.Colision.Superior, 650, 370, Plataforma.Diseño.Roca, Plataforma.Movimiento.Vertical, content);
		crearPlataforma(new Rectangle(2420, 460, 430, 30), content);
		crearPlataforma(new Rectangle(2420, 570, 430, 30), content);
		crearPlataforma(new Rectangle(230, 570, 250, 30), content);
		crearSubida(new Vector2(470f, 570f), 110, content, Plataforma.Colision.Total);
		crearPlataforma(new Rectangle(1020, 570, 330, 30), content);
		crearPlataformaMovediza(new Vector2(0f, 680f), Plataforma.Colision.Superior, 1000, 0, Plataforma.Diseño.Roca, Plataforma.Movimiento.Horizontal, content);
		crearPlataforma(new Rectangle(1340, 680, 440, 30), content);
		crearPlataforma(new Rectangle(2250, 680, 600, 30), content);
		crearPlataforma(new Rectangle(0, 790, 330, 30), content);
		crearPlataforma(new Rectangle(570, 790, 650, 30), content);
		crearPlataforma(new Rectangle(1800, 790, 450, 30), content);
		crearPlataforma(new Rectangle(0, 900, 2900, 30), content);
		crearSubida(new Vector2(1700f, 900f), 110, content, Plataforma.Colision.Total);
		fisica = new Fisica();
		menu = new MenuPersonajes();
		menu.CantidadControles = controles.Count;
		menu.cargar(content, this);
		cronometro = new Cronometro();
		cronometro.cargar(content);
		efecto = new Blur_Grayscale();
		efecto.cargar(content);
	}

	public override void actualizar(GameTime tiempo, List<Control> controles, Camara camara)
	{
		if (!menu.activo)
		{
			camara.activo = true;
			foreach (Plataforma plataforma in plataformas)
			{
				if (plataforma.Diseño_ != Plataforma.Diseño.Null)
				{
					plataforma.actualizar(tiempo);
				}
			}
			for (int i = 0; i < controles.Count; i++)
			{
				personajes[i].actualizar(tiempo, controles[i]);
			}
			foreach (Caja caja in cajas)
			{
				caja.actualizar(tiempo);
			}
			fisica.actualizar(this, tiempo);
			for (int i = 0; i < controles.Count; i++)
			{
				iconPer[i].actualizar(personajes[i]);
			}
			cronometro.actualizar(tiempo);
		}
		menu.actualizar(controles, this, camara);
		efecto.activo = menu.activo;
	}

	public override void dibujar(SpriteBatch sprite, Camara camara)
	{
		sprite.End();
		sprite.Begin();
		sprite.Draw(cielo, Vector2.Zero, Color.White);
		sprite.End();
		camara.dibujar(sprite);
		efecto.inicio(sprite, camara);
		sprite.Draw(scroll2, new Vector2((camara.Camara_.position.X - 250f) * 0.7f, (camara.Camara_.position.Y - 500f) * 0.7f), new Rectangle(0, 0, scroll2.Width, scroll2.Height), Color.White, 0f, Vector2.Zero, 4f, SpriteEffects.None, 1f);
		sprite.Draw(scroll1, new Vector2((camara.Camara_.position.X - 100f) * 0.5f, (camara.Camara_.position.Y - 600f) * 0.5f), new Rectangle(0, 0, scroll1.Width, scroll1.Height + 150), Color.White, 0f, Vector2.Zero, 2.9f, SpriteEffects.None, 1f);
		sprite.Draw(diseñoTrasero, posicionDiseño, new Rectangle(0, 0, 1280, 620), colorDiseño, 0f, Vector2.Zero, 2.25f, SpriteEffects.None, 1f);
		if (menu.PersonajesActivos)
		{
			foreach (Personaje personaje in personajes)
			{
				personaje.dibujar(sprite, camara);
			}
		}
		foreach (Moneda moneda in monedas)
		{
			moneda.dibujar(sprite);
		}
		foreach (Caja caja in cajas)
		{
			caja.dibujar(sprite);
		}
		foreach (Plataforma plataforma in plataformas)
		{
			if (plataforma.Diseño_ != Plataforma.Diseño.Null)
			{
				plataforma.dibujar(sprite);
			}
		}
		efecto.fin(sprite);
		if (menu.activo)
		{
			menu.dibujar(sprite, camara);
		}
	}
}
