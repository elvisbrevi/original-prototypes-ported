using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;
using Proyecto_Nutria.Iconos;

namespace Proyecto_Nutria;

internal class MenuPersonajes : Menu
{
	private List<bool> seleccionado = new List<bool>(4);

	private List<Texture2D> caras;

	private Texture2D cuadro;

	private Texture2D marco;

	private Texture2D fondo;

	private List<int> indices;

	private Color color;

	private SpriteFont fuente;

	private List<string> nombres;

	private List<Personaje> personajes;

	private ContentManager cont;

	private Vector2 posicionMarco = Vector2.Zero;

	private List<string> opciones = new List<string>();

	private int opcionSeleccionada;

	public override void cargar(ContentManager content, Nivel nivel)
	{
		opciones = new List<string>();
		opciones.Add("CONTINUAR");
		opciones.Add("REINICIAR");
		opciones.Add("SALIR");
		cont = content;
		fondo = content.Load<Texture2D>("Graficos/Menus/fondo");
		personajes = new List<Personaje>(2);
		personajes.Add(new Coipo());
		personajes.Add(new Pollo());
		foreach (Personaje personaje in personajes)
		{
			personaje.cargar(content);
		}
		seleccionado.Add(item: true);
		seleccionado.Add(item: true);
		nombres = new List<string>();
		nombres.Add("Coipo");
		nombres.Add("Pollo");
		fuente = content.Load<SpriteFont>("Fuentes/fuentePredeterminada");
		indices = new List<int>();
		indices.Add(0);
		indices.Add(5);
		indices.Add(2);
		indices.Add(7);
		marco = content.Load<Texture2D>("Graficos/Menus/MenuPersonajes/marco");
		cuadro = content.Load<Texture2D>("Graficos/Menus/MenuPersonajes/cuadro");
		caras = new List<Texture2D>();
		for (int i = 0; i < 8; i++)
		{
			Texture2D item = ((i >= 8) ? content.Load<Texture2D>("Graficos/Menus/MenuPersonajes/bloqueado") : content.Load<Texture2D>("Graficos/Menus/MenuPersonajes/cara" + (i + 1)));
			caras.Add(item);
		}
		nivel.personajes = new List<Personaje>();
		nivel.personajes.Add(new Pollo());
		nivel.personajes.Add(new Pollo());
		nivel.personajes[0] = personajes[indices[0]];
		nivel.personajes[0].Posicion = new Vector2(0f, 280f);
		nivel.personajes[0].limites = nivel.limites;
		nivel.personajes[0].cargar(cont);
		seleccionado[0] = true;
		nivel.personajes[1] = new Pollo();
		nivel.personajes[1].Posicion = new Vector2(2600f, 800f);
		nivel.personajes[1].limites = nivel.limites;
		nivel.personajes[1].cargar(cont);
		seleccionado[1] = true;
		base.PersonajesActivos = true;
		nivel.iconPer = new List<IconoPersonajes>();
		foreach (Personaje personaje2 in nivel.personajes)
		{
			IconoPersonajes iconoPersonajes = new IconoPersonajes();
			iconoPersonajes.cargar(cont);
			iconoPersonajes.personaje = personaje2;
			nivel.iconPer.Add(iconoPersonajes);
		}
	}

	public override void actualizar(List<Control> controles, Nivel nivel, Camara camara)
	{
		if (activo)
		{
			desplazarseEnMenu(controles);
			seleccionarOpciones(controles, nivel, camara);
		}
		foreach (Control controle in controles)
		{
			if (controle.Start_)
			{
				activo = true;
			}
		}
	}

	public override void dibujar(SpriteBatch sprite, Camara camara)
	{
		sprite.End();
		sprite.Begin();
		sprite.Draw(fondo, new Vector2(300f, 100f), Color.White);
		for (int i = 0; i < opciones.Count; i++)
		{
			if (i == opcionSeleccionada)
			{
				color = Color.Red;
			}
			else
			{
				color = Color.Black;
			}
			sprite.DrawString(fuente, opciones[i], new Vector2(450f, 170 + i * 50), color);
		}
		sprite.End();
		sprite.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, camara.Camara_.transform);
	}

	private void desplazarseEnMenu(List<Control> controles)
	{
		for (int i = 0; i < controles.Count; i++)
		{
			if (controles[i].Up_)
			{
				if (opcionSeleccionada == 0)
				{
					opcionSeleccionada = 2;
				}
				else
				{
					opcionSeleccionada--;
				}
			}
			if (controles[i].Down_)
			{
				if (opcionSeleccionada == 2)
				{
					opcionSeleccionada = 0;
				}
				else
				{
					opcionSeleccionada++;
				}
			}
		}
	}

	private void elegirPersonaje(Nivel nivel)
	{
		nivel.personajes = new List<Personaje>();
		nivel.personajes.Add(new Pollo());
		nivel.personajes.Add(new Pollo());
		nivel.personajes[0] = personajes[indices[0]];
		nivel.personajes[0].Posicion = new Vector2(0f, 280f);
		nivel.personajes[0].monedas = 0;
		nivel.personajes[0].limites = nivel.limites;
		nivel.personajes[0].cargar(cont);
		seleccionado[0] = true;
		nivel.personajes[1] = new Pollo();
		nivel.personajes[1].Posicion = new Vector2(2600f, 800f);
		nivel.personajes[1].monedas = 0;
		nivel.personajes[1].limites = nivel.limites;
		nivel.personajes[1].cargar(cont);
		seleccionado[1] = true;
		base.PersonajesActivos = true;
		nivel.iconPer = new List<IconoPersonajes>();
		foreach (Personaje personaje in nivel.personajes)
		{
			IconoPersonajes iconoPersonajes = new IconoPersonajes();
			iconoPersonajes.cargar(cont);
			iconoPersonajes.personaje = personaje;
			nivel.iconPer.Add(iconoPersonajes);
		}
	}

	private void seleccionarOpciones(List<Control> controles, Nivel nivel, Camara camara)
	{
		foreach (Control controle in controles)
		{
			switch (opcionSeleccionada)
			{
			case 0:
				if (controle.A_)
				{
					activo = false;
					opcionSeleccionada = 0;
				}
				break;
			case 1:
				if (controle.A_)
				{
					elegirPersonaje(nivel);
					activo = false;
					opcionSeleccionada = 0;
					camara.Camara_.position.X = 0f;
					camara.Camara_.scale = 0.36f;
				}
				break;
			case 2:
				if (controle.A_)
				{
					elegirPersonaje(nivel);
					activo = false;
					opcionSeleccionada = 0;
				}
				break;
			}
		}
	}

	private void recorrerMenu(Control control)
	{
		if (control.Down_)
		{
			if (opcionSeleccionada == opciones.Count - 1)
			{
				opcionSeleccionada = 0;
			}
			else
			{
				opcionSeleccionada++;
			}
		}
		if (control.Up_)
		{
			if (opcionSeleccionada == 0)
			{
				opcionSeleccionada = opciones.Count - 1;
			}
			else
			{
				opcionSeleccionada--;
			}
		}
	}
}
