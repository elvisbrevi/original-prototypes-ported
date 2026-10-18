using Microsoft.DirectX.DirectInput;
using Microsoft.Xna.Framework;

namespace Iman.Controles;

public abstract class Control
{
	protected string nombre;

	protected bool A;

	protected bool B;

	protected bool Start;

	protected bool L;

	protected bool R;

	protected bool Up;

	protected bool Down;

	protected bool Left;

	protected bool Right;

	protected bool aHold;

	protected bool bHold;

	protected bool startHold;

	protected bool lHold;

	protected bool rHold;

	protected bool upHold;

	protected bool downHold;

	protected bool leftHold;

	protected bool rightHold;

	protected Vector2 posicionPuntero;

	protected bool clickUnoHold;

	protected bool clickUno;

	protected bool clickDosHold;

	protected bool clickDos;

	public bool ClickDosHold
	{
		get
		{
			return clickDosHold;
		}
		set
		{
			clickDosHold = value;
		}
	}

	public bool ClickUno
	{
		get
		{
			return clickUno;
		}
		set
		{
			clickUno = value;
		}
	}

	public bool ClickUnoHold
	{
		get
		{
			return clickUnoHold;
		}
		set
		{
			clickUnoHold = value;
		}
	}

	public bool ClickDos
	{
		get
		{
			return clickDos;
		}
		set
		{
			clickDos = value;
		}
	}

	public Vector2 PosicionPuntero
	{
		get
		{
			return posicionPuntero;
		}
		set
		{
			posicionPuntero = value;
		}
	}

	public string Nombre
	{
		get
		{
			return nombre;
		}
		set
		{
			nombre = value;
		}
	}

	public bool RightHold
	{
		get
		{
			return rightHold;
		}
		set
		{
			rightHold = value;
		}
	}

	public bool DownHold
	{
		get
		{
			return downHold;
		}
		set
		{
			downHold = value;
		}
	}

	public bool UpHold
	{
		get
		{
			return upHold;
		}
		set
		{
			upHold = value;
		}
	}

	public bool RHold
	{
		get
		{
			return rHold;
		}
		set
		{
			rHold = value;
		}
	}

	public bool LHold
	{
		get
		{
			return lHold;
		}
		set
		{
			lHold = value;
		}
	}

	public bool StartHold
	{
		get
		{
			return startHold;
		}
		set
		{
			startHold = value;
		}
	}

	public bool BHold
	{
		get
		{
			return bHold;
		}
		set
		{
			bHold = value;
		}
	}

	public bool AHold
	{
		get
		{
			return aHold;
		}
		set
		{
			aHold = value;
		}
	}

	public bool LeftHold
	{
		get
		{
			return leftHold;
		}
		set
		{
			leftHold = value;
		}
	}

	public bool Right_
	{
		get
		{
			return Right;
		}
		set
		{
			Right = value;
		}
	}

	public bool Left_
	{
		get
		{
			return Left;
		}
		set
		{
			Left = value;
		}
	}

	public bool Down_
	{
		get
		{
			return Down;
		}
		set
		{
			Down = value;
		}
	}

	public bool Up_
	{
		get
		{
			return Up;
		}
		set
		{
			Up = value;
		}
	}

	public bool L_
	{
		get
		{
			return L;
		}
		set
		{
			L = value;
		}
	}

	public bool Start_
	{
		get
		{
			return Start;
		}
		set
		{
			Start = value;
		}
	}

	public bool B_
	{
		get
		{
			return B;
		}
		set
		{
			B = value;
		}
	}

	public bool A_
	{
		get
		{
			return A;
		}
		set
		{
			A = value;
		}
	}

	public bool R_
	{
		get
		{
			return R;
		}
		set
		{
			R = value;
		}
	}

	public static int verificarJoystick()
	{
		DeviceList devices = Manager.GetDevices((DeviceClass)4, (EnumDevicesFlags)1);
		return devices.Count;
	}

	public abstract void cargar(Game1 game);

	public abstract void actualizar(Game1 game);
}
