using System.Collections.Generic;
using EAAE.Niveles;
using Engine.Audios;
using Engine.Datos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Audio;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE;

public class Game1 : Game
{
	private GraphicsDeviceManager graphics;

	private SpriteBatch spriteBatch;

	private Control control;

	private List<Nivel> niveles;

	public int nivelActivo = 0;

	public List<Sonido> musica = new List<Sonido>();

	public Game1()
	{
		graphics = new GraphicsDeviceManager(this);
		base.Content.RootDirectory = "Content";
		base.Window.Title = "The Electric Eel Attack";
	}

	protected override void Initialize()
	{
		base.Initialize();
	}

	protected override void LoadContent()
	{
		spriteBatch = new SpriteBatch(base.GraphicsDevice);
		List<string> list = new List<string>();
		list.Add("cancion1");
		list.Add("cancion2");
		foreach (string item in list)
		{
			Sonido sonido = new Sonido();
			sonido.cargar(base.Content, item);
			musica.Add(sonido);
		}
		musica[0].instanciaSonido.Volume = 0.8f;
		musica[1].instanciaSonido.Volume = 0.3f;
		control = new Control();
		niveles = new List<Nivel>();
		niveles.Add(new Titulo());
		niveles.Add(new Nivel1());
		niveles.Add(new Nivel2());
		niveles.Add(new Nivel3());
		niveles.Add(new Nivel4());
		niveles[nivelActivo].cargar(base.Content, graphics);
		Pantalla.iniciar(Escenario.Ancho, Escenario.Alto, graphics, fullScreen: false);
	}

	public void cambiarNivel(int indice)
	{
		nivelActivo = indice;
		niveles[nivelActivo].cargar(base.Content, graphics);
	}

	protected override void UnloadContent()
	{
	}

	protected override void Update(GameTime gameTime)
	{
		if (nivelActivo < 3)
		{
			if (musica[0].instanciaSonido.State != SoundState.Playing)
			{
				musica[1].stop();
				musica[0].Activo = true;
			}
		}
		else if (musica[1].instanciaSonido.State != SoundState.Playing)
		{
			musica[0].stop();
			musica[1].Activo = true;
		}
		foreach (Sonido item in musica)
		{
			item.play();
			item.Activo = false;
		}
		control.iniciar();
		if (control.PressEsc)
		{
			graphics.ToggleFullScreen();
		}
		niveles[nivelActivo].actualizar(gameTime, control, this);
		control.finalizar();
		base.Update(gameTime);
	}

	protected override void Draw(GameTime gameTime)
	{
		niveles[nivelActivo].dibujar(spriteBatch, graphics);
		base.Draw(gameTime);
	}
}
