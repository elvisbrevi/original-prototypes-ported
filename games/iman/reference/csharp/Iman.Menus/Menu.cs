using System.Collections.Generic;
using Iman.Camaras;
using Iman.Controles;
using Iman.Escenarios;
using Iman.Shaders;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Menus;

internal class Menu
{
	private Post_Processing shader;

	private Color color = Color.White;

	private List<string> opciones = new List<string>();

	private int opcionSeleccionada;

	private string opcion;

	private Texture2D titulo;

	private SpriteFont fuenteOpciones;

	private Escenario mapa;

	public bool activo;

	public bool exit;

	public bool creditos;

	private ContentManager content;

	private Texture2D cortina;

	private Color colorCortina = Color.Black;

	private Color colorCreditos = new Color(Color.White, 0);

	public void cargar(ContentManager content, Camera2D camara)
	{
		this.content = content;
		activo = true;
		opciones = new List<string>();
		opciones.Add("Play");
		opciones.Add("Quit");
		fuenteOpciones = content.Load<SpriteFont>("Textos/fuente");
		titulo = content.Load<Texture2D>("Graficos/Menu/titulo");
		cortina = content.Load<Texture2D>("Graficos/Menu/cortina");
		mapa = new Escenario_01();
		mapa.cargar(content, camara);
		shader = new NonPhotoRealistic();
		shader.cargar(content);
		shader.activo = true;
	}

	public void actualizar(Control control, GameTime tiempo, ContentManager content, Camera2D camara)
	{
		if (control.Start_ && !activo)
		{
			activo = true;
		}
		if (activo)
		{
			shader.actualizar(tiempo);
			recorrerMenu(control);
			seleccionarOpciones(control, content, camara);
			return;
		}
		if (mapa.activo)
		{
			if (colorCortina.A > 1)
			{
				colorCortina.A--;
			}
			mapa.actualizar(tiempo, camara, control);
			return;
		}
		if (colorCortina.A < 254)
		{
			colorCortina.A++;
			return;
		}
		if (mapa.ID == 0)
		{
			mapa = new Escenario_02();
			mapa.cargar(content, camara);
			mapa.activo = true;
		}
		if (mapa.ID == 1)
		{
			creditos = true;
			if (colorCreditos.A < 254)
			{
				colorCreditos.A++;
			}
		}
	}

	public void dibujar(SpriteBatch sprite, Camera2D camara, GraphicsDeviceManager graphics)
	{
		if (activo)
		{
			sprite.End();
			sprite.Begin(SpriteBlendMode.None, SpriteSortMode.Immediate, SaveStateMode.SaveState);
			shader.iniciar(sprite, graphics);
			sprite.Draw(titulo, Vector2.Zero, Color.White);
			shader.finalizar(sprite, graphics);
			sprite.End();
			sprite.Begin();
			for (int i = 0; i < opciones.Count; i++)
			{
				string value = opciones[i];
				if (opcionSeleccionada == i)
				{
					color = Color.Green;
					opciones[i] = "*" + opciones[i];
				}
				else
				{
					color = Color.Yellow;
					opciones[i] = value;
				}
				sprite.DrawString(fuenteOpciones, opciones[i], new Vector2(250f, 150 + i * 50), color);
				opciones[i] = value;
			}
		}
		else
		{
			mapa.dibujar(sprite, camara, graphics);
			sprite.Draw(cortina, new Rectangle((int)camara.position.X, (int)camara.position.Y, 640, 480), colorCortina);
			if (creditos)
			{
				sprite.DrawString(fuenteOpciones, "FIN...", new Vector2(camara.position.X + 260f, camara.position.Y + 200f), colorCreditos);
			}
		}
	}

	private void seleccionarOpciones(Control control, ContentManager content, Camera2D camara)
	{
		if (control.A_)
		{
			switch (opcion)
			{
			case "Play":
				mapa = new Escenario_01();
				mapa.cargar(content, camara);
				mapa.activo = true;
				opciones = new List<string>();
				opciones.Add("Resume");
				opciones.Add("Menu");
				colorCortina.A = byte.MaxValue;
				opcionSeleccionada = 0;
				activo = false;
				break;
			case "Quit":
				exit = true;
				break;
			case "Resume":
				mapa.activo = true;
				activo = false;
				break;
			case "Menu":
				mapa.activo = false;
				opciones = new List<string>();
				opciones.Add("Play");
				opciones.Add("Quit");
				opcionSeleccionada = 0;
				activo = true;
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
		opcion = opciones[opcionSeleccionada];
	}
}
