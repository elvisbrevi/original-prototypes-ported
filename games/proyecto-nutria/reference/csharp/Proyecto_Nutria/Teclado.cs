using Microsoft.DirectX.DirectInput;
using Microsoft.Xna.Framework.Input;

namespace Proyecto_Nutria;

internal class Teclado : Control
{
	private KeyboardState teclaAnterior;

	public override void cargar(Game1 game)
	{
		DeviceList devices = Manager.GetDevices((DeviceClass)4, (EnumDevicesFlags)1);
		cantidadJoysticks = devices.Count;
	}

	public override void actualizar()
	{
		KeyboardState state = Keyboard.GetState();
		if (state.IsKeyDown(Keys.Z) && teclaAnterior.IsKeyUp(Keys.Z))
		{
			A_ = true;
		}
		else
		{
			A_ = false;
		}
		if (state.IsKeyDown(Keys.Z))
		{
			AHold = true;
		}
		else
		{
			AHold = false;
		}
		if (state.IsKeyDown(Keys.X) && teclaAnterior.IsKeyUp(Keys.X))
		{
			B_ = true;
		}
		else
		{
			B_ = false;
		}
		if (state.IsKeyDown(Keys.X))
		{
			BHold = true;
		}
		else
		{
			BHold = false;
		}
		if (state.IsKeyDown(Keys.Enter) && teclaAnterior.IsKeyUp(Keys.Enter))
		{
			Start_ = true;
		}
		else
		{
			Start_ = false;
		}
		if (state.IsKeyDown(Keys.Enter))
		{
			startHold = true;
		}
		else
		{
			startHold = false;
		}
		if (state.IsKeyDown(Keys.A) && teclaAnterior.IsKeyUp(Keys.A))
		{
			L_ = true;
		}
		else
		{
			L_ = false;
		}
		if (state.IsKeyDown(Keys.A))
		{
			lHold = true;
		}
		else
		{
			lHold = false;
		}
		if (state.IsKeyDown(Keys.S) && teclaAnterior.IsKeyUp(Keys.S))
		{
			R_ = true;
		}
		else
		{
			R_ = false;
		}
		if (state.IsKeyDown(Keys.S))
		{
			rHold = true;
		}
		else
		{
			rHold = false;
		}
		if (state.IsKeyDown(Keys.Up) && teclaAnterior.IsKeyUp(Keys.Up))
		{
			Up_ = true;
		}
		else
		{
			Up_ = false;
		}
		if (state.IsKeyDown(Keys.Up))
		{
			upHold = true;
		}
		else
		{
			upHold = false;
		}
		if (state.IsKeyDown(Keys.Down) && teclaAnterior.IsKeyUp(Keys.Down))
		{
			Down_ = true;
		}
		else
		{
			Down_ = false;
		}
		if (state.IsKeyDown(Keys.Down))
		{
			downHold = true;
		}
		else
		{
			downHold = false;
		}
		if (state.IsKeyDown(Keys.Left) && teclaAnterior.IsKeyUp(Keys.Left))
		{
			Left_ = true;
		}
		else
		{
			Left_ = false;
		}
		if (state.IsKeyDown(Keys.Left))
		{
			LeftHold = true;
		}
		else
		{
			LeftHold = false;
		}
		if (state.IsKeyDown(Keys.Right) && teclaAnterior.IsKeyUp(Keys.Right))
		{
			Right_ = true;
		}
		else
		{
			Right_ = false;
		}
		if (state.IsKeyDown(Keys.Right))
		{
			RightHold = true;
		}
		else
		{
			RightHold = false;
		}
		teclaAnterior = state;
	}
}
