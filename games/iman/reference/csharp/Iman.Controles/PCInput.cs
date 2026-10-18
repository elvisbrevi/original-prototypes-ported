using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;

namespace Iman.Controles;

public class PCInput : Control
{
	private KeyboardState teclaAnterior;

	private MouseState mouseAnterior;

	public override void cargar(Game1 game)
	{
	}

	public override void actualizar(Game1 game)
	{
		MouseState state = Mouse.GetState();
		base.PosicionPuntero = new Vector2(state.X, state.Y);
		if (state.LeftButton == ButtonState.Pressed)
		{
			clickUnoHold = true;
		}
		else
		{
			clickUnoHold = false;
		}
		if (state.RightButton == ButtonState.Pressed)
		{
			clickDosHold = true;
		}
		else
		{
			clickDosHold = false;
		}
		if (state.LeftButton == ButtonState.Pressed && mouseAnterior.LeftButton == ButtonState.Released)
		{
			clickUno = true;
		}
		else
		{
			clickUno = false;
		}
		if (state.RightButton == ButtonState.Pressed && mouseAnterior.RightButton == ButtonState.Released)
		{
			clickDos = true;
		}
		else
		{
			clickDos = true;
		}
		mouseAnterior = state;
		KeyboardState state2 = Keyboard.GetState();
		if (state2.IsKeyDown(Keys.Z) && teclaAnterior.IsKeyUp(Keys.Z))
		{
			A = true;
		}
		else
		{
			A = false;
		}
		if (state2.IsKeyDown(Keys.Z))
		{
			aHold = true;
		}
		else
		{
			aHold = false;
		}
		if (state2.IsKeyDown(Keys.X) && teclaAnterior.IsKeyUp(Keys.X))
		{
			B = true;
		}
		else
		{
			B = false;
		}
		if (state2.IsKeyDown(Keys.X))
		{
			bHold = true;
		}
		else
		{
			bHold = false;
		}
		if (state2.IsKeyDown(Keys.Enter) && teclaAnterior.IsKeyUp(Keys.Enter))
		{
			Start = true;
		}
		else
		{
			Start = false;
		}
		if (state2.IsKeyDown(Keys.Enter))
		{
			startHold = true;
		}
		else
		{
			startHold = false;
		}
		if (state2.IsKeyDown(Keys.A) && teclaAnterior.IsKeyUp(Keys.A))
		{
			L = true;
		}
		else
		{
			L = false;
		}
		if (state2.IsKeyDown(Keys.A))
		{
			lHold = true;
		}
		else
		{
			lHold = false;
		}
		if (state2.IsKeyDown(Keys.S) && teclaAnterior.IsKeyUp(Keys.S))
		{
			R = true;
		}
		else
		{
			R = false;
		}
		if (state2.IsKeyDown(Keys.S))
		{
			rHold = true;
		}
		else
		{
			rHold = false;
		}
		if (state2.IsKeyDown(Keys.Up) && teclaAnterior.IsKeyUp(Keys.Up))
		{
			Up = true;
		}
		else
		{
			Up = false;
		}
		if (state2.IsKeyDown(Keys.Up))
		{
			upHold = true;
		}
		else
		{
			upHold = false;
		}
		if (state2.IsKeyDown(Keys.Down) && teclaAnterior.IsKeyUp(Keys.Down))
		{
			Down = true;
		}
		else
		{
			Down = false;
		}
		if (state2.IsKeyDown(Keys.Down))
		{
			downHold = true;
		}
		else
		{
			downHold = false;
		}
		if (state2.IsKeyDown(Keys.Left) && teclaAnterior.IsKeyUp(Keys.Left))
		{
			Left = true;
		}
		else
		{
			Left = false;
		}
		if (state2.IsKeyDown(Keys.Left))
		{
			leftHold = true;
		}
		else
		{
			leftHold = false;
		}
		if (state2.IsKeyDown(Keys.Right) && teclaAnterior.IsKeyUp(Keys.Right))
		{
			Right = true;
		}
		else
		{
			Right = false;
		}
		if (state2.IsKeyDown(Keys.Right))
		{
			rightHold = true;
		}
		else
		{
			rightHold = false;
		}
		teclaAnterior = state2;
	}
}
