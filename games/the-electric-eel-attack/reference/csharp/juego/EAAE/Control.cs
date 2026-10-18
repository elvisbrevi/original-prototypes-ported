using Microsoft.Xna.Framework.Input;

namespace EAAE;

public class Control
{
	private KeyboardState teclaAnterior;

	private KeyboardState tecla;

	private MouseState mouseAnterior;

	private MouseState mouse;

	private int mouseX;

	private int mouseY;

	private bool pressZ;

	private bool holdZ;

	private bool pressX;

	private bool holdX;

	private bool pressStart;

	private bool pressEsc;

	private bool leftClick;

	private bool holdLeftClick;

	private bool rightClick;

	private bool holdRightClick;

	private bool left;

	private bool holdLeft;

	private bool right;

	private bool holdRight;

	private bool up;

	private bool holdUp;

	private bool down;

	private bool holdDown;

	private bool mouseQuieto;

	private bool reiniciar;

	public bool Reiniciar => reiniciar;

	public bool MouseQuieto => mouseQuieto;

	public int MouseY => mouseY;

	public int MouseX => mouseX;

	public bool HoldRightClick => holdRightClick;

	public bool HoldLeftClick => holdLeftClick;

	public bool RightClick => rightClick;

	public bool LeftClick => leftClick;

	public bool PressEsc => pressEsc;

	public bool PressZ => pressZ;

	public bool HoldPressZ => holdZ;

	public bool PressX => pressX;

	public bool HoldPressX => holdX;

	public bool PressStart => pressStart;

	public bool PressLeft => left;

	public bool HoldPressLeft => holdLeft;

	public bool PressRight => right;

	public bool HoldPressRight => holdRight;

	public bool PressUp => up;

	public bool HoldPressUp => holdUp;

	public bool PressDown => down;

	public bool HoldPressDown => holdDown;

	public void iniciar()
	{
		tecla = Keyboard.GetState();
		mouse = Mouse.GetState();
		mouseX = mouse.X;
		mouseY = mouse.Y;
		if (mouseAnterior.X == mouse.X && mouseAnterior.Y == mouse.Y)
		{
			mouseQuieto = true;
		}
		else
		{
			mouseQuieto = false;
		}
		if (mouse.LeftButton == ButtonState.Pressed && mouseAnterior.LeftButton == ButtonState.Released)
		{
			leftClick = true;
		}
		else
		{
			leftClick = false;
		}
		if (mouse.LeftButton == ButtonState.Pressed)
		{
			holdLeftClick = true;
		}
		else
		{
			holdLeftClick = false;
		}
		if (mouse.RightButton == ButtonState.Pressed && mouseAnterior.RightButton == ButtonState.Released)
		{
			rightClick = true;
		}
		else
		{
			rightClick = false;
		}
		if (mouse.RightButton == ButtonState.Pressed)
		{
			holdRightClick = true;
		}
		else
		{
			holdRightClick = false;
		}
		if (tecla.IsKeyDown(Keys.R) && teclaAnterior.IsKeyUp(Keys.R))
		{
			reiniciar = true;
		}
		else
		{
			reiniciar = false;
		}
		if (tecla.IsKeyDown(Keys.Z) && teclaAnterior.IsKeyUp(Keys.Z))
		{
			pressZ = true;
		}
		else
		{
			pressZ = false;
		}
		if (tecla.IsKeyDown(Keys.Z))
		{
			holdZ = true;
		}
		else
		{
			holdZ = false;
		}
		if (tecla.IsKeyDown(Keys.X) && teclaAnterior.IsKeyUp(Keys.X))
		{
			pressX = true;
		}
		else
		{
			pressX = false;
		}
		if (tecla.IsKeyDown(Keys.X))
		{
			holdX = true;
		}
		else
		{
			holdX = false;
		}
		if (tecla.IsKeyDown(Keys.Enter) && teclaAnterior.IsKeyUp(Keys.Enter))
		{
			pressStart = true;
		}
		else
		{
			pressStart = false;
		}
		if (tecla.IsKeyDown(Keys.Escape) && teclaAnterior.IsKeyUp(Keys.Escape))
		{
			pressEsc = true;
		}
		else
		{
			pressEsc = false;
		}
		if (tecla.IsKeyDown(Keys.Left) && teclaAnterior.IsKeyUp(Keys.Left))
		{
			left = true;
		}
		else
		{
			left = false;
		}
		if (tecla.IsKeyDown(Keys.Left))
		{
			holdLeft = true;
		}
		else
		{
			holdLeft = false;
		}
		if (tecla.IsKeyDown(Keys.Right) && teclaAnterior.IsKeyUp(Keys.Right))
		{
			right = true;
		}
		else
		{
			right = false;
		}
		if (tecla.IsKeyDown(Keys.Right))
		{
			holdRight = true;
		}
		else
		{
			holdRight = false;
		}
		if (tecla.IsKeyDown(Keys.Up) && teclaAnterior.IsKeyUp(Keys.Up))
		{
			up = true;
		}
		else
		{
			up = false;
		}
		if (tecla.IsKeyDown(Keys.Up))
		{
			holdUp = true;
		}
		else
		{
			holdUp = false;
		}
		if (tecla.IsKeyDown(Keys.Down) && teclaAnterior.IsKeyUp(Keys.Down))
		{
			down = true;
		}
		else
		{
			down = false;
		}
		if (tecla.IsKeyDown(Keys.Down))
		{
			holdDown = true;
		}
		else
		{
			holdDown = false;
		}
	}

	public void finalizar()
	{
		teclaAnterior = tecla;
		mouseAnterior = mouse;
	}
}
