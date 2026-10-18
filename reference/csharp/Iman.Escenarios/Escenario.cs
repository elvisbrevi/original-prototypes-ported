using System.Collections.Generic;
using Iman.Camaras;
using Iman.Controles;
using Iman.Fisicas;
using Iman.Graficos;
using Iman.Objetos;
using Iman.Shaders;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Escenarios;

internal abstract class Escenario
{
	protected List<Ramas> ramas;

	protected Post_Processing shader;

	protected Fisica fisica;

	public Rectangle limites = default(Rectangle);

	public List<Objeto> objetos = new List<Objeto>();

	protected List<Grafico> graficos = new List<Grafico>();

	public bool activo;

	public int ID;

	public abstract void cargar(ContentManager content, Camera2D camara);

	public abstract void actualizar(GameTime tiempo, Camera2D camara, Control control);

	public abstract void dibujar(SpriteBatch sprite, Camera2D camara, GraphicsDeviceManager graphics);

	protected void crearPlataforma(Rectangle rectangulo, Objeto.Diseño diseño, Objeto.Colision colision, ContentManager content)
	{
		Plataforma plataforma = new Plataforma();
		plataforma.cuerpo = rectangulo;
		plataforma.colision = colision;
		plataforma.diseño = diseño;
		plataforma.posicion = new Vector2(rectangulo.X, rectangulo.Y);
		plataforma.cargar(content);
		objetos.Add(plataforma);
	}

	protected void crearPlataformaIman(Vector2 posicion, Objeto.Diseño diseño, Objeto.Colision colision, ContentManager content)
	{
		Plataforma plataforma = new Plataforma();
		plataforma.colision = colision;
		plataforma.diseño = diseño;
		plataforma.posicion = posicion;
		plataforma.gravedad = true;
		plataforma.cargar(content);
		objetos.Add(plataforma);
	}

	protected void crearPlataformaMovediza(Vector2 posicion, ContentManager content, Objeto.Diseño diseño, Objeto.Movimiento movimiento, int rangoMovimiento, int velocidad)
	{
		Plataforma plataforma = new Plataforma();
		plataforma.colision = Objeto.Colision.Superior;
		plataforma.diseño = diseño;
		plataforma.posicion = posicion;
		plataforma.movimiento = movimiento;
		plataforma.velocidad = velocidad;
		if (movimiento == Objeto.Movimiento.Horizontal)
		{
			plataforma.posicionInicial = (int)posicion.X;
			plataforma.posicionFinal = (int)posicion.X + rangoMovimiento;
		}
		else
		{
			plataforma.posicionInicial = (int)posicion.Y;
			plataforma.posicionFinal = (int)posicion.Y + rangoMovimiento;
		}
		plataforma.cargar(content);
		objetos.Add(plataforma);
	}

	protected void crearSubida(Vector2 posicion, int largo, int inclinacion, ContentManager content)
	{
		for (int i = 0; i < largo; i++)
		{
			Plataforma plataforma = new Plataforma();
			plataforma.cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, 5, 5);
			plataforma.colision = Objeto.Colision.Total;
			plataforma.diseño = Objeto.Diseño.NULL;
			plataforma.posicion = new Vector2(posicion.X, posicion.Y);
			plataforma.cargar(content);
			objetos.Add(plataforma);
			posicion.X += 5f;
			posicion.Y -= inclinacion;
		}
	}

	protected void crearBajada(Vector2 posicion, int largo, int inclinacion, ContentManager content)
	{
		for (int i = 0; i < largo; i++)
		{
			Plataforma plataforma = new Plataforma();
			plataforma.cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, 5, 5);
			plataforma.colision = Objeto.Colision.Total;
			plataforma.diseño = Objeto.Diseño.NULL;
			plataforma.posicion = new Vector2(posicion.X, posicion.Y);
			plataforma.cargar(content);
			objetos.Add(plataforma);
			posicion.X += 5f;
			posicion.Y += inclinacion;
		}
	}

	protected void crearGrafico(Grafico.Diseño diseño, Grafico.Tipo tipo, Vector2 posicion, float zoom, ContentManager content)
	{
		Grafico grafico = new Background();
		switch (tipo)
		{
		case Grafico.Tipo.Background:
			grafico = new Background();
			break;
		case Grafico.Tipo.Cielo:
			grafico = new Cielo();
			break;
		case Grafico.Tipo.Front:
			grafico = new Front();
			break;
		case Grafico.Tipo.Suelo:
			grafico = new Suelo();
			break;
		}
		grafico.posicion = posicion;
		grafico.zoom = zoom;
		grafico.diseño = diseño;
		grafico.tipo = tipo;
		grafico.cargar(content);
		graficos.Add(grafico);
	}

	protected void crearNiño(Vector2 posicion, ContentManager content)
	{
		Niño niño = new Niño();
		niño.posicion = posicion;
		niño.cargar(content);
		objetos.Add(niño);
	}

	protected void moverCamara(Camera2D camara)
	{
		camara.position.X = objetos[objetos.Count - 1].origen.X - 320f;
		camara.position.X = MathHelper.Clamp(camara.position.X, camara.limites.Left, camara.limites.Right);
		camara.position.Y = objetos[objetos.Count - 1].origen.Y - 240f;
		camara.position.Y = MathHelper.Clamp(camara.position.Y, camara.limites.Top, camara.limites.Bottom);
		objetos[objetos.Count - 1].posicion.X = MathHelper.Clamp(objetos[objetos.Count - 1].posicion.X, limites.Left, limites.Right - objetos[objetos.Count - 1].cuerpo.Width);
	}
}
