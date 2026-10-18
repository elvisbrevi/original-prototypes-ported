using System;
using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Microsoft.Xna.Framework.Input;
using Proyecto_Nutria.Iconos;

namespace Proyecto_Nutria;

public class Game1 : Game
{
	private GraphicsDeviceManager graphics;

	private SpriteBatch spriteBatch;

	private Camara camara;

	private Viewport areaJuego;

	private Viewport areaDefault;

	private List<Control> controles;

	private Control c;

	private Nivel nivel;

	public static bool pause;

	private Texture2D textura;

	public Game1()
	{
		graphics = new GraphicsDeviceManager(this);
		base.Content.RootDirectory = "Content";
		base.Window.Title = "Proyecto Nutria (Version Prototipo)";
		graphics.PreferredBackBufferHeight = 720;
		graphics.PreferredBackBufferWidth = 1280;
	}

	protected override void Initialize()
	{
		camara = new Camara();
		camara.inicializar(this);
		camara.Camara_.position = new Vector2(500f, 500f);
		base.Initialize();
	}

	protected override void LoadContent()
	{
		textura = base.Content.Load<Texture2D>("Graficos/Plataformas/null");
		spriteBatch = new SpriteBatch(base.GraphicsDevice);
		c = new Teclado();
		controles = new List<Control>();
		if (c.cantidadJoysticks == 0)
		{
			controles.Add(new Teclado());
		}
		else
		{
			for (int i = 0; i < c.cantidadJoysticks; i++)
			{
				controles.Add(new JoystickGenerico());
			}
			controles.Add(new Teclado());
		}
		foreach (Control controle in controles)
		{
			controle.cargar(this);
		}
		areaDefault = base.GraphicsDevice.Viewport;
		areaJuego = base.GraphicsDevice.Viewport;
		areaJuego.X = 100;
		areaJuego.Y = 100;
		areaJuego.Width = 1080;
		areaJuego.Height = 520;
		nivel = new Nivel_1();
		nivel.cargar(base.Content, controles);
	}

	protected override void Update(GameTime gameTime)
	{
		if (Keyboard.GetState().IsKeyDown(Keys.Escape))
		{
			Exit();
		}
		foreach (Control controle in controles)
		{
			controle.actualizar();
		}
		if (!pause)
		{
			camara.actualizar(nivel);
			nivel.actualizar(gameTime, controles, camara);
		}
		base.Update(gameTime);
	}

	protected override void Draw(GameTime gameTime)
	{
		base.GraphicsDevice.Viewport = areaDefault;
		base.GraphicsDevice.Clear(Color.CornflowerBlue);
		spriteBatch.Begin();
		spriteBatch.Draw(nivel.marco, Vector2.Zero, Color.White);
		try
		{
			for (int i = 0; i < nivel.iconPer.Count; i++)
			{
				if (i == 0)
				{
					nivel.iconPer[i].dibujar(spriteBatch, IconoPersonajes.Posicion.SuperiorIzquierda);
				}
				else
				{
					nivel.iconPer[i].dibujar(spriteBatch, IconoPersonajes.Posicion.SuperiorDerecha);
				}
			}
		}
		catch (NullReferenceException)
		{
		}
		spriteBatch.End();
		camara.dibujar(spriteBatch);
		base.GraphicsDevice.Viewport = areaJuego;
		base.GraphicsDevice.Clear(Color.Black);
		nivel.dibujar(spriteBatch, camara);
		spriteBatch.End();
		base.Draw(gameTime);
	}

	protected override void UnloadContent()
	{
	}

	private void graphics_PreparingDeviceSettings(object sender, PreparingDeviceSettingsEventArgs e)
	{
		foreach (DisplayMode supportedDisplayMode in GraphicsAdapter.DefaultAdapter.SupportedDisplayModes)
		{
			if (supportedDisplayMode.Width == 1920 && supportedDisplayMode.Height == 1080)
			{
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferFormat = supportedDisplayMode.Format;
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferHeight = supportedDisplayMode.Height;
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferWidth = supportedDisplayMode.Width;
				e.GraphicsDeviceInformation.PresentationParameters.FullScreenRefreshRateInHz = supportedDisplayMode.RefreshRate;
				e.GraphicsDeviceInformation.PresentationParameters.IsFullScreen = true;
			}
			else
			{
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferFormat = supportedDisplayMode.Format;
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferHeight = 720;
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferWidth = 1280;
				e.GraphicsDeviceInformation.PresentationParameters.FullScreenRefreshRateInHz = supportedDisplayMode.RefreshRate;
				e.GraphicsDeviceInformation.PresentationParameters.IsFullScreen = true;
			}
		}
	}
}
