using Microsoft.DirectX.DirectInput;

namespace Proyecto_Nutria;

internal abstract class Control
{
	public int cantidadJoysticks;

	public string nombre;

	public bool A_;

	public bool B_;

	public bool Start_;

	public bool L_;

	public bool R_;

	public bool Up_;

	public bool Down_;

	public bool Left_;

	public bool Right_;

	public bool AHold;

	public bool BHold;

	public bool startHold;

	public bool lHold;

	public bool rHold;

	public bool upHold;

	public bool downHold;

	public bool LeftHold;

	public bool RightHold;

	public Control()
	{
		DeviceList devices = Manager.GetDevices((DeviceClass)4, (EnumDevicesFlags)1);
		cantidadJoysticks = devices.Count;
	}

	public abstract void cargar(Game1 game);

	public abstract void actualizar();
}
