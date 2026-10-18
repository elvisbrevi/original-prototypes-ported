using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Microsoft.Xna.Framework.Input;

namespace Engine.Datos;

public static class Pantalla
{
	private static GraphicsDeviceManager _Device = null;

	public static int _Width = 800;

	public static int _Height = 600;

	public static int _VWidth = 800;

	public static int _VHeight = 600;

	private static Matrix _ScaleMatrix;

	private static bool _FullScreen = false;

	private static bool _dirtyMatrix = true;

	public static float Tamaño;

	public static float Inclinacion;

	public static Vector2 Posicion;

	public static void iniciar(int ancho, int alto, GraphicsDeviceManager g, bool fullScreen)
	{
		Init(ref g);
		SetVirtualResolution(ancho, alto);
		SetResolution(ancho, alto, fullScreen);
	}

	public static void Init(ref GraphicsDeviceManager device)
	{
		_Width = device.PreferredBackBufferWidth;
		_Height = device.PreferredBackBufferHeight;
		_Device = device;
		_dirtyMatrix = true;
		ApplyResolutionSettings();
	}

	public static Matrix getTransformationMatrix()
	{
		if (_dirtyMatrix)
		{
			RecreateScaleMatrix();
		}
		return _ScaleMatrix;
	}

	public static int getHeight()
	{
		return _Height;
	}

	public static int getWidth()
	{
		return _Width;
	}

	public static int getVHeight()
	{
		return _VHeight;
	}

	public static int getVWidth()
	{
		return _VWidth;
	}

	public static void SetResolution(int Width, int Height, bool FullScreen)
	{
		_Width = Width;
		_Height = Height;
		_FullScreen = FullScreen;
		ApplyResolutionSettings();
	}

	public static void reset()
	{
		_dirtyMatrix = true;
	}

	public static void SetVirtualResolution(int Width, int Height)
	{
		_VWidth = Width;
		_VHeight = Height;
		_dirtyMatrix = true;
	}

	private static void ApplyResolutionSettings()
	{
		if (!_FullScreen)
		{
			if (_Width <= GraphicsAdapter.DefaultAdapter.CurrentDisplayMode.Width && _Height <= GraphicsAdapter.DefaultAdapter.CurrentDisplayMode.Height)
			{
				_Device.PreferredBackBufferWidth = _Width;
				_Device.PreferredBackBufferHeight = _Height;
				_Device.IsFullScreen = _FullScreen;
				_Device.ApplyChanges();
			}
		}
		else
		{
			foreach (DisplayMode supportedDisplayMode in GraphicsAdapter.DefaultAdapter.SupportedDisplayModes)
			{
				if (supportedDisplayMode.Width == _Width && supportedDisplayMode.Height == _Height)
				{
					_Device.PreferredBackBufferWidth = _Width;
					_Device.PreferredBackBufferHeight = _Height;
					_Device.IsFullScreen = _FullScreen;
					_Device.ApplyChanges();
				}
			}
		}
		_dirtyMatrix = true;
		_Width = _Device.PreferredBackBufferWidth;
		_Height = _Device.PreferredBackBufferHeight;
	}

	public static void BeginDraw()
	{
		FullViewport();
		_Device.GraphicsDevice.Clear(Color.Black);
		ResetViewport();
		_Device.GraphicsDevice.Clear(Color.CornflowerBlue);
	}

	private static void RecreateScaleMatrix()
	{
		_dirtyMatrix = false;
		_ScaleMatrix = Matrix.CreateTranslation(new Vector3(0f - Posicion.X, 0f - Posicion.Y, 0f)) * Matrix.CreateScale(Tamaño) * Matrix.CreateRotationZ(Inclinacion);
	}

	public static void actualizar()
	{
		if (Keyboard.GetState().IsKeyDown(Keys.Q) && Inclinacion > -50f)
		{
			Inclinacion -= 0.01f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.E) && Inclinacion < 50f)
		{
			Inclinacion += 0.01f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.Z) && Tamaño < 1f)
		{
			Tamaño += 0.01f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.C) && Tamaño > 0.01f)
		{
			Tamaño -= 0.01f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.A))
		{
			Posicion.X -= 10f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.D))
		{
			Posicion.X += 10f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.S))
		{
			Posicion.Y += 10f;
			reset();
		}
		if (Keyboard.GetState().IsKeyDown(Keys.W))
		{
			Posicion.Y -= 10f;
			reset();
		}
	}

	public static void FullViewport()
	{
		Viewport viewport = default(Viewport);
		int x = (viewport.Y = 0);
		viewport.X = x;
		viewport.Width = _Width;
		viewport.Height = _Height;
		_Device.GraphicsDevice.Viewport = viewport;
	}

	public static float getVirtualAspectRatio()
	{
		return (float)_VWidth / (float)_VHeight;
	}

	public static void ResetViewport()
	{
		float virtualAspectRatio = getVirtualAspectRatio();
		int num = _Device.PreferredBackBufferWidth;
		int num2 = (int)((float)num / virtualAspectRatio + 0.5f);
		bool flag = false;
		if (num2 > _Device.PreferredBackBufferHeight)
		{
			num2 = _Device.PreferredBackBufferHeight;
			num = (int)((float)num2 * virtualAspectRatio + 0.5f);
			flag = true;
		}
		Viewport viewport = new Viewport
		{
			X = _Device.PreferredBackBufferWidth / 2 - num / 2,
			Y = _Device.PreferredBackBufferHeight / 2 - num2 / 2,
			Width = num,
			Height = num2,
			MinDepth = 0f,
			MaxDepth = 1f
		};
		if (flag)
		{
			_dirtyMatrix = true;
		}
		_Device.GraphicsDevice.Viewport = viewport;
	}
}
