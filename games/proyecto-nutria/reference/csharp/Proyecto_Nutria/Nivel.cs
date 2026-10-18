using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;
using Proyecto_Nutria.Iconos;
using Proyecto_Nutria.Shaders;

namespace Proyecto_Nutria;

internal abstract class Nivel
{
	public Post_Processing efecto;

	public List<Moneda> monedas = new List<Moneda>();

	public List<IconoPersonajes> iconPer;

	protected Texture2D diseñoTrasero;

	protected Vector2 posicionDiseño;

	protected Texture2D scroll1;

	protected Texture2D scroll2;

	protected Color colorDiseño;

	public Texture2D marco;

	public Texture2D cielo;

	public Rectangle limites;

	public List<Personaje> personajes;

	public Menu menu;

	public List<Caja> cajas;

	public List<Plataforma> plataformas;

	public Fisica fisica;

	public Cronometro cronometro;

	public abstract void cargar(ContentManager content, List<Control> controles);

	public abstract void actualizar(GameTime tiempo, List<Control> controles, Camara camara);

	public abstract void dibujar(SpriteBatch sprite, Camara camara);

	protected void crearCaja(Vector2 posicion, ContentManager content)
	{
		Caja caja = new Caja();
		caja.posicion = posicion;
		caja.cargar(content);
		cajas.Add(caja);
	}

	protected void crearPlataforma(Rectangle cuerpo, ContentManager con)
	{
		Plataforma plataforma = new Plataforma();
		plataforma.Posicion = new Vector2(cuerpo.X, cuerpo.Y);
		plataforma.Cuerpo = cuerpo;
		plataforma.cargar(con);
		plataformas.Add(plataforma);
	}

	protected void crearPlataforma(Rectangle cuerpo, Plataforma.Colision colision, ContentManager con)
	{
		Plataforma plataforma = new Plataforma();
		plataforma.Posicion = new Vector2(cuerpo.X, cuerpo.Y);
		plataforma.Cuerpo = cuerpo;
		plataforma.Colision_ = colision;
		plataforma.cargar(con);
		plataformas.Add(plataforma);
	}

	protected void crearPlataformaMovediza(Vector2 posicion, Plataforma.Colision colision, int max, int min, Plataforma.Diseño diseño, Plataforma.Movimiento movimiento, ContentManager con)
	{
		Plataforma plataforma = new Plataforma();
		plataforma.Posicion = posicion;
		plataforma.Colision_ = colision;
		plataforma.Diseño_ = diseño;
		plataforma.Movimiento_ = movimiento;
		plataforma.posicionInicial = min;
		plataforma.posicionFinal = max;
		plataforma.velocidadMovimiento = 3f;
		plataforma.cargar(con);
		plataformas.Add(plataforma);
	}

	protected void crearBajada(Vector2 posicion, int cantidad, ContentManager content, Plataforma.Colision colision)
	{
		Vector2 posicion2 = posicion;
		for (int i = 0; i < cantidad; i++)
		{
			Plataforma plataforma = new Plataforma();
			plataforma.Posicion = posicion2;
			plataforma.Cuerpo = new Rectangle((int)posicion2.X, (int)posicion2.Y, 1, 1);
			posicion2.X += 1f;
			posicion2.Y += 1f;
			plataforma.Colision_ = colision;
			plataforma.cargar(content);
			plataformas.Add(plataforma);
		}
	}

	protected void crearSubida(Vector2 posicion, int cantidad, ContentManager content, Plataforma.Colision colision)
	{
		Vector2 posicion2 = posicion;
		for (int i = 0; i < cantidad; i++)
		{
			Plataforma plataforma = new Plataforma();
			plataforma.Posicion = posicion2;
			plataforma.Cuerpo = new Rectangle((int)posicion2.X, (int)posicion2.Y, 1, 1);
			posicion2.X += 1f;
			posicion2.Y -= 1f;
			plataforma.Colision_ = colision;
			plataforma.cargar(content);
			plataformas.Add(plataforma);
		}
	}

	protected void generar(Vector2 posicion, Vector2 direccion, GameTime tiempo, int id)
	{
		monedas[id].direccion = direccion;
		monedas[id].posicion = posicion;
		monedas[id].posicionInicial = posicion;
		monedas[id].activa = true;
	}
}
