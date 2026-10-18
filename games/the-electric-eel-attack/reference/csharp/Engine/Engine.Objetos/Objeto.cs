using System.Collections.Generic;
using Engine.Audios;
using Engine.Shaders;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Objetos;

public abstract class Objeto
{
	public enum Colision
	{
		Ninguna,
		Total,
		Lateral,
		Superior,
		Inferior,
		Pixel
	}

	public enum Tipo
	{
		Personaje,
		Grafico,
		Plataforma,
		Particulas,
		Texto
	}

	public enum Lugar
	{
		Fondo,
		Medio,
		Frente
	}

	public enum Shader
	{
		Ninguno,
		Filter,
		Negative,
		Blur_Grayscale,
		ColorSketch,
		MonoSketch,
		Brightness,
		TopFade,
		LeftFade,
		TopLeftFade,
		Fishbowl,
		Blur,
		Grayscale
	}

	public Vector2 Direccion;

	public string Texto;

	public SpriteFont Fuente;

	public Colision Colision_;

	public string Nombre;

	public string NombreTextura;

	public SpriteBlendMode blendMode = SpriteBlendMode.AlphaBlend;

	public bool Movimento;

	public bool Seleccionado;

	public bool Animado;

	public bool Gravedad;

	public bool colisionSuperior;

	public bool colisionInferior;

	public bool colisionDerecha;

	public bool colisionIzquierda;

	public bool repetirScroll;

	public bool fueraDeCamara;

	public bool noShaderCamara;

	public float ScrollHorizontal;

	public float ScrollVertical;

	public float Tamaño;

	public float Inclinacion;

	public float FuerzaGravedad;

	public float Velocidad;

	public int EstadoSeleccionado;

	public int CantidadParticulas;

	public Tipo Tipo_;

	public Vector2 Posicion;

	public Vector2 Destino;

	public Vector2 Inicio;

	public Vector2 Origen;

	public Matrix Transform;

	public GraphicsDevice GraphicsDevice;

	public Texture2D Textura;

	public Color Color_;

	public SpriteEffects SpriteEffect_;

	public Lugar Lugar_;

	public Rectangle Cuerpo = default(Rectangle);

	public Rectangle bodyData = default(Rectangle);

	public List<Estado> Estados = new List<Estado>();

	public List<Sonido> Sonidos = new List<Sonido>();

	public Shader Shader_;

	public Post_Processing PostProcessing = null;

	public Color[] TextureData;

	public float respawn;

	public bool idaX = true;

	public bool idaY = true;

	private bool fadeIn;

	private bool fadeOut = true;

	private int aditivo = 0;

	public bool ColorRandom { get; set; }

	public bool TamañoRandom { get; set; }

	public bool DireccionRandom { get; set; }

	public bool Activo { get; set; }

	public abstract void cargar(ContentManager content);

	public abstract void actualizar(GameTime tiempo);

	public abstract void dibujar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics);

	public void cargarSonidos(ContentManager content)
	{
		foreach (Sonido sonido in Sonidos)
		{
			sonido.cargar(content, sonido.Nombre);
		}
	}

	public void actualizarSonidos()
	{
		foreach (Sonido sonido in Sonidos)
		{
			sonido.play();
			sonido.Activo = false;
		}
	}

	protected void mover()
	{
		if (Inicio.X != Destino.X)
		{
			if (idaX)
			{
				Posicion.X += Velocidad;
				if (Posicion.X > Destino.X)
				{
					idaX = false;
				}
			}
			else
			{
				Posicion.X -= Velocidad;
				if (Posicion.X < Inicio.X)
				{
					idaX = true;
				}
			}
		}
		if (Inicio.Y == Destino.Y)
		{
			return;
		}
		if (idaY)
		{
			Posicion.Y += Velocidad;
			if (Posicion.Y > Destino.Y)
			{
				idaY = false;
			}
		}
		else
		{
			Posicion.Y -= Velocidad;
			if (Posicion.Y < Inicio.Y)
			{
				idaY = true;
			}
		}
	}

	public void cambiarEstado(int indice)
	{
		EstadoSeleccionado = indice;
		if (Estados[EstadoSeleccionado].Numero > 1f)
		{
			Estados[EstadoSeleccionado].Cuadro.X = 0;
		}
	}

	public void Fade(byte velocidad, GameTime tiempo)
	{
		if (fadeOut)
		{
			if ((double)(int)Color_.A > tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad)
			{
				Color_.A -= (byte)(tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad);
			}
			else
			{
				fadeOut = false;
				fadeIn = true;
			}
		}
		if (fadeIn)
		{
			if ((double)(int)Color_.A < 254.0 - tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad)
			{
				Color_.A += (byte)(tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad);
				return;
			}
			fadeIn = false;
			fadeOut = true;
		}
	}

	public void FadeOut(byte velocidad, GameTime tiempo)
	{
		if ((double)(int)Color_.A > tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad)
		{
			Color_.A -= (byte)(tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad);
			return;
		}
		Color_.A = 0;
		Activo = false;
	}

	public void FadeIn(byte velocidad, GameTime tiempo)
	{
		if (!Activo)
		{
			Activo = true;
		}
		if ((double)(int)Color_.A < 254.0 - tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad)
		{
			Color_.A += (byte)(tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)(int)velocidad);
		}
		else
		{
			Color_.A = byte.MaxValue;
		}
	}

	public void atenuarAditivo(int velocidad, GameTime tiempo)
	{
		if (blendMode == SpriteBlendMode.Additive)
		{
			if ((double)aditivo > tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)velocidad)
			{
				aditivo -= (byte)(tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)velocidad);
			}
			else
			{
				blendMode = SpriteBlendMode.AlphaBlend;
			}
		}
		if (blendMode == SpriteBlendMode.AlphaBlend)
		{
			if ((double)aditivo < tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)velocidad)
			{
				aditivo += (byte)(tiempo.ElapsedGameTime.TotalSeconds * 100.0 * (double)velocidad);
			}
			else
			{
				blendMode = SpriteBlendMode.Additive;
			}
		}
	}

	public void reproducir(string nombre)
	{
		foreach (Sonido sonido in Sonidos)
		{
			if (sonido.Nombre == nombre)
			{
				sonido.Activo = true;
			}
		}
	}

	public void reproducir(string nombre, Vector2 emisor, Vector2 oyente)
	{
		foreach (Sonido sonido in Sonidos)
		{
			if (sonido.Nombre == nombre)
			{
				float num = MathHelper.Distance(emisor.X, oyente.X);
				float num2 = MathHelper.Distance(emisor.Y, oyente.Y);
				float num3 = (num + num2) * 0.001f;
				if (sonido.Volumen - num3 <= 1f && sonido.Volumen - num3 >= 0f)
				{
					sonido.instanciaSonido.Volume = sonido.Volumen - num3;
				}
				else
				{
					sonido.instanciaSonido.Volume = 0f;
				}
				sonido.Activo = true;
			}
		}
	}

	public void stop(string nombre)
	{
		foreach (Sonido sonido in Sonidos)
		{
			if (sonido.Nombre == nombre)
			{
				sonido.Activo = false;
			}
		}
	}
}
