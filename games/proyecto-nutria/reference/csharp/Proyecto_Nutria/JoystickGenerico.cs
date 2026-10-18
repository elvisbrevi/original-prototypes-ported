using Microsoft.DirectX.DirectInput;

namespace Proyecto_Nutria;

internal class JoystickGenerico : Control
{
	private Device Joystick;

	private JoystickState state;

	private byte[] botones;

	private byte[] botonesAnteriores;

	private int stick;

	private int stickAnterior;

	public override void cargar(Game1 game)
	{
		//IL_003e: Unknown result type (might be due to invalid IL or missing references)
		//IL_0043: Unknown result type (might be due to invalid IL or missing references)
		//IL_004d: Unknown result type (might be due to invalid IL or missing references)
		//IL_0057: Expected O, but got Unknown
		//IL_00b5: Unknown result type (might be due to invalid IL or missing references)
		//IL_00ba: Unknown result type (might be due to invalid IL or missing references)
		//IL_00ea: Unknown result type (might be due to invalid IL or missing references)
		DeviceList devices = Manager.GetDevices((DeviceClass)4, (EnumDevicesFlags)1);
		cantidadJoysticks = devices.Count;
		if (devices.Count <= 0)
		{
			return;
		}
		foreach (DeviceInstance item in devices)
		{
			DeviceInstance val = item;
			Joystick = new Device(((DeviceInstance)(ref val)).InstanceGuid);
			if (!(Joystick != (Device)null))
			{
				continue;
			}
			Joystick.SetCooperativeLevel(game.Window.Handle, (CooperativeLevelFlags)9);
			Joystick.Properties.AxisModeAbsolute = true;
			foreach (DeviceObjectInstance @object in Joystick.Objects)
			{
				DeviceObjectInstance val2 = @object;
				if ((((DeviceObjectInstance)(ref val2)).ObjectId & 3) != 0)
				{
					Joystick.Properties.SetRange((ParameterHow)2, ((DeviceObjectInstance)(ref val2)).ObjectId, new InputRange(-1000, 1000));
					Joystick.Properties.SetDeadZone((ParameterHow)2, ((DeviceObjectInstance)(ref val2)).ObjectId, 5000);
				}
			}
		}
		Joystick.Acquire();
		nombre = Joystick.Properties.ProductName;
	}

	public override void actualizar()
	{
		//IL_0008: Unknown result type (might be due to invalid IL or missing references)
		//IL_000d: Unknown result type (might be due to invalid IL or missing references)
		state = Joystick.CurrentJoystickState;
		botones = ((JoystickState)(ref state)).GetButtons();
		stick = ((JoystickState)(ref state)).GetPointOfView()[0];
		if (botones[0] == 128 && botonesAnteriores[0] == 0)
		{
			A_ = true;
		}
		else
		{
			A_ = false;
		}
		if (botones[0] == 128)
		{
			AHold = true;
		}
		else
		{
			AHold = false;
		}
		if (botones[1] == 128 && botonesAnteriores[1] == 0)
		{
			B_ = true;
		}
		else
		{
			B_ = false;
		}
		if (botones[1] == 128)
		{
			BHold = true;
		}
		else
		{
			BHold = false;
		}
		if (botones[9] == 128 && botonesAnteriores[9] == 0)
		{
			Start_ = true;
		}
		else
		{
			Start_ = false;
		}
		if (botones[9] == 128)
		{
			startHold = true;
		}
		else
		{
			startHold = false;
		}
		if (botones[4] == 128 && botonesAnteriores[4] == 0)
		{
			L_ = true;
		}
		else
		{
			L_ = false;
		}
		if (botones[4] == 128)
		{
			lHold = true;
		}
		else
		{
			lHold = false;
		}
		if (botones[5] == 128 && botonesAnteriores[5] == 0)
		{
			R_ = true;
		}
		else
		{
			R_ = false;
		}
		if (botones[5] == 128)
		{
			rHold = true;
		}
		else
		{
			rHold = false;
		}
		if (stick == 0 && stickAnterior != 0)
		{
			Up_ = true;
		}
		else
		{
			Up_ = false;
		}
		if (stick == 0)
		{
			upHold = true;
		}
		else
		{
			upHold = false;
		}
		if (stick == 18000 && stickAnterior != 18000)
		{
			Down_ = true;
		}
		else
		{
			Down_ = false;
		}
		if (stick == 18000)
		{
			downHold = true;
		}
		else
		{
			downHold = false;
		}
		if ((stick == 27000 && stickAnterior != 27000) || (stick == 31500 && stickAnterior != 31500) || (stick == 22500 && stickAnterior != 22500))
		{
			Left_ = true;
		}
		else
		{
			Left_ = false;
		}
		if (stick == 27000 || stick == 31500 || stick == 22500)
		{
			LeftHold = true;
		}
		else
		{
			LeftHold = false;
		}
		if ((stick == 9000 && stickAnterior != 9000) || (stick == 4500 && stickAnterior != 4500) || (stick == 13500 && stickAnterior != 13500))
		{
			Right_ = true;
		}
		else
		{
			Right_ = false;
		}
		if (stick == 9000 || stick == 4500 || stick == 13500)
		{
			RightHold = true;
		}
		else
		{
			RightHold = false;
		}
		botonesAnteriores = botones;
		stickAnterior = stick;
	}
}
