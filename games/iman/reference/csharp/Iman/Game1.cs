using Iman.Camaras;
using Iman.Controles;
using Iman.Menus;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Microsoft.Xna.Framework.Input;

namespace Iman;

public class Game1 : Game
{
	private GraphicsDeviceManager graphics;

	private SpriteBatch spriteBatch;

	private Camera2D camara;

	private Control control;

	private Menu menu;

	public Game1()
	{
		graphics = new GraphicsDeviceManager(this);
		base.Content.RootDirectory = "Content";
		base.Window.Title = "IMAN (Prototipo)";
		graphics.PreferredBackBufferHeight = 480;
		graphics.PreferredBackBufferWidth = 640;
	}

	protected override void Initialize()
	{
		camara = new Camera2D(this, manualcamera: true);
		base.Components.Add(camara);
		base.IsMouseVisible = true;
		base.Initialize();
	}

	protected override void LoadContent()
	{
		spriteBatch = new SpriteBatch(base.GraphicsDevice);
		if (Control.verificarJoystick() > 0)
		{
			control = new JoystickNormal();
			control.cargar(this);
		}
		else
		{
			control = new PCInput();
			control.cargar(this);
		}
		menu = new Menu();
		menu.cargar(base.Content, camara);
	}

	protected override void UnloadContent()
	{
	}

	protected override void Update(GameTime gameTime)
	{
		if (Keyboard.GetState().IsKeyDown(Keys.Escape))
		{
			Exit();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.F4))
		{
			graphics.ToggleFullScreen();
		}
		control.actualizar(this);
		menu.actualizar(control, gameTime, base.Content, camara);
		if (menu.exit)
		{
			Exit();
		}
		base.Update(gameTime);
	}

	protected override void Draw(GameTime gameTime)
	{
		base.GraphicsDevice.Clear(Color.Black);
		spriteBatch.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, camara.transform);
		menu.dibujar(spriteBatch, camara, graphics);
		spriteBatch.End();
		base.Draw(gameTime);
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
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferHeight = 768;
				e.GraphicsDeviceInformation.PresentationParameters.BackBufferWidth = 1024;
				e.GraphicsDeviceInformation.PresentationParameters.FullScreenRefreshRateInHz = supportedDisplayMode.RefreshRate;
				e.GraphicsDeviceInformation.PresentationParameters.IsFullScreen = true;
			}
		}
	}
}
