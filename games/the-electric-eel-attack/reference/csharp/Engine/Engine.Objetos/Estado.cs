using Microsoft.Xna.Framework;

namespace Engine.Objetos;

public class Estado
{
	public enum Variacion
	{
		Ninguna,
		Shader,
		Rotacion,
		Tamaño
	}

	public Rectangle Cuadro;

	public Variacion Variacion_;

	public float FPS;

	public float Frama;

	public float Numero;

	public float MinVariacion;

	public float MaxVariacion;

	public float VelocidadVariacion;

	public string Nombre;

	public bool NoRepetir;

	private bool variarMas = true;

	public void animar(GameTime tiempo, Objeto objeto)
	{
		if (!NoRepetir)
		{
			if (Variacion_ != Variacion.Ninguna)
			{
				aplicarVariacion(objeto);
			}
			if (Frama > (float)Cuadro.Width)
			{
				Frama = 0f;
				if ((float)Cuadro.X >= (float)Cuadro.Width * Numero - (float)Cuadro.Width)
				{
					Cuadro.X = -Cuadro.Width;
				}
				Cuadro.X += Cuadro.Width;
			}
			Frama += FPS * ((float)tiempo.ElapsedGameTime.TotalSeconds * 10f);
			return;
		}
		if (Variacion_ != Variacion.Ninguna)
		{
			aplicarVariacion(objeto);
		}
		if (Frama > (float)Cuadro.Width)
		{
			Frama = 0f;
			if ((float)Cuadro.X < (float)Cuadro.Width * Numero - (float)Cuadro.Width)
			{
				Cuadro.X += Cuadro.Width;
			}
		}
		Frama += FPS * ((float)tiempo.ElapsedGameTime.TotalSeconds * 10f);
	}

	protected void aplicarVariacion(Objeto objeto)
	{
		switch (Variacion_)
		{
		case Variacion.Shader:
			if (objeto.Shader_ == Objeto.Shader.Brightness || objeto.Shader_ == Objeto.Shader.Fishbowl || objeto.Shader_ == Objeto.Shader.Grayscale)
			{
				if (variarMas)
				{
					objeto.PostProcessing.intensidad += VelocidadVariacion;
					if (objeto.PostProcessing.intensidad > MaxVariacion)
					{
						variarMas = false;
					}
				}
				else
				{
					objeto.PostProcessing.intensidad -= VelocidadVariacion;
					if (objeto.PostProcessing.intensidad < MinVariacion)
					{
						variarMas = true;
					}
				}
			}
			else
			{
				if (objeto.Shader_ != Objeto.Shader.Filter)
				{
					break;
				}
				if (variarMas)
				{
					objeto.PostProcessing.burn += VelocidadVariacion;
					if (objeto.PostProcessing.burn > MaxVariacion)
					{
						variarMas = false;
					}
				}
				else
				{
					objeto.PostProcessing.burn -= VelocidadVariacion;
					if (objeto.PostProcessing.burn < MinVariacion)
					{
						variarMas = true;
					}
				}
			}
			break;
		case Variacion.Rotacion:
			if (MaxVariacion == 0f && MinVariacion == 0f)
			{
				objeto.Inclinacion += VelocidadVariacion;
			}
			else if (variarMas)
			{
				objeto.Inclinacion += VelocidadVariacion;
				if (objeto.Inclinacion > MaxVariacion)
				{
					variarMas = false;
				}
			}
			else
			{
				objeto.Inclinacion -= VelocidadVariacion;
				if (objeto.Inclinacion < MinVariacion)
				{
					variarMas = true;
				}
			}
			break;
		case Variacion.Tamaño:
			if (variarMas)
			{
				objeto.Tamaño += VelocidadVariacion;
				if (objeto.Tamaño > MaxVariacion)
				{
					variarMas = false;
				}
			}
			else
			{
				objeto.Tamaño -= VelocidadVariacion;
				if (objeto.Tamaño < MinVariacion)
				{
					variarMas = true;
				}
			}
			break;
		}
	}
}
