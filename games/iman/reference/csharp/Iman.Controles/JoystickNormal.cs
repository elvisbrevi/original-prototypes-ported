using Microsoft.DirectX.DirectInput;

namespace Iman.Controles;

public class JoystickNormal : Control
{
	private Device Joystick;

	private JoystickState state;

	private byte[] botones;

	private byte[] botonesAnteriores;

	private int stick;

	private int stickAnterior;

	public override void cargar(Game1 game)
	{
		//IL_0032: Unknown result type (might be due to invalid IL or missing references)
		//IL_0037: Unknown result type (might be due to invalid IL or missing references)
		//IL_0041: Unknown result type (might be due to invalid IL or missing references)
		//IL_004b: Expected O, but got Unknown
		//IL_00a9: Unknown result type (might be due to invalid IL or missing references)
		//IL_00ae: Unknown result type (might be due to invalid IL or missing references)
		//IL_00de: Unknown result type (might be due to invalid IL or missing references)
		DeviceList devices = Manager.GetDevices((DeviceClass)4, (EnumDevicesFlags)1);
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

	public override void actualizar(Game1 game)
	{
		//IL_0009: Unknown result type (might be due to invalid IL or missing references)
		//IL_000e: Unknown result type (might be due to invalid IL or missing references)
		try
		{
			state = Joystick.CurrentJoystickState;
		}
		catch (InputLostException)
		{
		}
		botones = ((JoystickState)(ref state)).GetButtons();
		stick = ((JoystickState)(ref state)).GetPointOfView()[0];
		if (botones[0] == 128 && botonesAnteriores[0] == 0)
		{
			A = true;
		}
		else
		{
			A = false;
		}
		if (botones[0] == 128)
		{
			aHold = true;
		}
		else
		{
			aHold = false;
		}
		if (botones[1] == 128 && botonesAnteriores[1] == 0)
		{
			B = true;
		}
		else
		{
			B = false;
		}
		if (botones[1] == 128)
		{
			bHold = true;
		}
		else
		{
			bHold = false;
		}
		if (botones[9] == 128 && botonesAnteriores[9] == 0)
		{
			Start = true;
		}
		else
		{
			Start = false;
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
			L = true;
		}
		else
		{
			L = false;
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
			R = true;
		}
		else
		{
			R = false;
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
			Up = true;
		}
		else
		{
			Up = false;
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
			Down = true;
		}
		else
		{
			Down = false;
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
			Left = true;
		}
		else
		{
			Left = false;
		}
		if (stick == 27000 || stick == 31500 || stick == 22500)
		{
			leftHold = true;
		}
		else
		{
			leftHold = false;
		}
		if ((stick == 9000 && stickAnterior != 9000) || (stick == 4500 && stickAnterior != 4500) || (stick == 13500 && stickAnterior != 13500))
		{
			Right = true;
		}
		else
		{
			Right = false;
		}
		if (stick == 9000 || stick == 4500 || stick == 13500)
		{
			rightHold = true;
		}
		else
		{
			rightHold = false;
		}
		botonesAnteriores = botones;
		stickAnterior = stick;
	}
}
