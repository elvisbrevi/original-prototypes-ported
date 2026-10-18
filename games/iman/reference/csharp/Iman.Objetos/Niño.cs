using Iman.Camaras;
using Iman.Controles;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Audio;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Objetos;

internal class Niño : Objeto
{
	private Rectangle cuadro = default(Rectangle);

	private Color color = Color.White;

	private float zoom = 1f;

	private SpriteEffects direccion = SpriteEffects.None;

	private float distanciaSalto;

	private float limiteSalto;

	private float velocidadSalto;

	private float alturaSalto;

	private float maxFrameRate;

	private float frameRate;

	private Vector2 framePerSeconds;

	private Texture2D texturaIman;

	private Rectangle cuadroIman;

	private Vector2 frameIman = new Vector2(60f, 0f);

	private SpriteEffects direccionIman;

	private float rotacionIman;

	private Vector2 posicionIman;

	private AudioEngine engine;

	private WaveBank wave;

	private SoundBank sound;

	public override void cargar(ContentManager content)
	{
		tipoObjeto = TipoObjeto.Personaje;
		diseño = Diseño._01;
		gravedad = true;
		textura = content.Load<Texture2D>("Graficos/Personajes/niño");
		cuadro = new Rectangle(0, 0, textura.Width / 8, textura.Height / 2);
		cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, (int)((float)cuadro.Width * zoom), (int)((float)cuadro.Height * zoom));
		origen = new Vector2(cuerpo.Width / 2, cuerpo.Height / 2);
		velocidad = 3;
		alturaSalto = 7f;
		textureData = new Color[textura.Width * textura.Height];
		textura.GetData(textureData);
		framePerSeconds = new Vector2(60f, 0f);
		maxFrameRate = textura.Width - cuadro.Width;
		rangoIman = 50f;
		texturaIman = content.Load<Texture2D>("Graficos/Personajes/iman");
		cuadroIman = new Rectangle(0, 0, 5, 10);
		engine = new AudioEngine("content/Audio/Audio.xgs");
		wave = new WaveBank(engine, "content/Audio/wave bank.xwb");
		sound = new SoundBank(engine, "content/Audio/sound bank.xsb");
	}

	public override void actualizar(GameTime tiempo, Control control)
	{
		mover(control);
		saltar(control);
		cambiarEstados(control);
		animar(tiempo, control);
		usarMagnetismo(control);
		engine.Update();
	}

	public override void dibujar(SpriteBatch sprite, Camera2D camara)
	{
		if (usandoIman)
		{
			dibujarIman(sprite, camara);
		}
		sprite.Draw(textura, new Vector2(posicion.X + origen.X, posicion.Y + origen.Y), cuadro, color, rotacion, origen, zoom, direccion, 1f);
	}

	private void usarMagnetismo(Control control)
	{
		if (control.AHold)
		{
			usandoIman = true;
		}
		else
		{
			usandoIman = false;
		}
		switch (orientacion)
		{
		case Orientacion.Izquierda:
			iman = new Rectangle(cuerpo.Right + 10, (int)posicion.Y, (int)rangoIman, cuerpo.Height);
			posicionIman = new Vector2(cuerpo.Right, posicion.Y);
			direccionIman = SpriteEffects.None;
			rotacionIman = 0f;
			break;
		case Orientacion.Derecha:
			iman = new Rectangle((int)(posicion.X - rangoIman) - 10, (int)posicion.Y, (int)rangoIman, cuerpo.Height);
			posicionIman = new Vector2((float)cuerpo.Left - rangoIman, posicion.Y);
			direccionIman = SpriteEffects.FlipHorizontally;
			rotacionIman = 0f;
			break;
		case Orientacion.Arriba:
			iman = new Rectangle((int)posicion.X, (int)(posicion.Y - rangoIman) - 10, cuerpo.Width, (int)rangoIman);
			posicionIman = new Vector2(posicion.X, posicion.Y);
			rotacionIman = -1.57f;
			break;
		case Orientacion.Abajo:
			break;
		}
	}

	private void mover(Control control)
	{
		if (control.LeftHold)
		{
			posicion.X -= velocidad;
			direccion = SpriteEffects.None;
			orientacion = Orientacion.Izquierda;
		}
		if (control.RightHold)
		{
			posicion.X += velocidad;
			direccion = SpriteEffects.FlipHorizontally;
			orientacion = Orientacion.Derecha;
		}
		if (control.UpHold)
		{
			orientacion = Orientacion.Arriba;
		}
		if (control.DownHold)
		{
			orientacion = Orientacion.Abajo;
		}
		cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, cuadro.Width, cuadro.Height);
		origen = new Vector2(posicion.X + (float)(cuerpo.Width / 2), posicion.Y + (float)(cuerpo.Height / 2));
	}

	private void saltar(Control control)
	{
		if (control.B_)
		{
			if (colisionSuperior)
			{
				sound.PlayCue("jump");
			}
			limiteSalto = posicion.Y - 600f;
			distanciaSalto = posicion.Y;
			velocidadSalto = alturaSalto;
		}
		if (control.BHold && estado != Estado.Cayendo && !colisionInferior)
		{
			if (distanciaSalto > limiteSalto)
			{
				distanciaSalto -= 8f;
				estado = Estado.Saltando;
			}
			if (estado == Estado.Saltando && posicion.Y > distanciaSalto)
			{
				posicion.Y -= velocidadSalto;
				if (velocidadSalto >= 0.2f)
				{
					velocidadSalto -= 0.2f;
				}
				if (velocidadSalto < 0.3f)
				{
					estado = Estado.Parado;
				}
			}
		}
		else
		{
			distanciaSalto = 0f;
			velocidadSalto = 0f;
			estado = Estado.Cayendo;
		}
	}

	private void cambiarEstados(Control control)
	{
		if (estado != Estado.Saltando)
		{
			if (control.LeftHold || control.RightHold)
			{
				estado = Estado.Corriendo;
				colisionEscalera = false;
			}
			if (!control.RightHold && !control.LeftHold)
			{
				estado = Estado.Parado;
			}
		}
		if (!colisionSuperior && estado != Estado.Saltando)
		{
			estado = Estado.Cayendo;
		}
	}

	private void animar(GameTime gameTime, Control control)
	{
		switch (estado)
		{
		case Estado.Saltando:
			cuadro.Y = cuadro.Height;
			cuadro.X = cuadro.Width * 5;
			break;
		case Estado.Cayendo:
			cuadro.Y = cuadro.Height;
			cuadro.X = cuadro.Width * 6;
			break;
		case Estado.Parado:
			cuadro.Y = cuadro.Height;
			cuadro.X = cuadro.Width * 3;
			break;
		case Estado.Corriendo:
			if (frameRate > (float)cuadro.Width)
			{
				frameRate = 0f;
				if ((float)cuadro.X >= maxFrameRate)
				{
					cuadro.X = -cuadro.Width;
				}
				cuadro.Y = 0;
				cuadro.X += cuadro.Width;
			}
			frameRate += framePerSeconds.X * ((float)gameTime.ElapsedGameTime.TotalSeconds * 10f);
			break;
		}
		if (frameIman.X > 5f)
		{
			if (cuadroIman.X == 45)
			{
				cuadroIman.X = 0;
			}
			else
			{
				cuadroIman.X += 5;
			}
			frameIman.X = 0f;
		}
		frameIman.X += (float)gameTime.ElapsedGameTime.TotalSeconds * 100f;
	}

	private void dibujarIman(SpriteBatch sprite, Camera2D camara)
	{
		sprite.End();
		sprite.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, camara.transform);
		sprite.Draw(texturaIman, new Rectangle((int)posicionIman.X, (int)posicionIman.Y, (int)rangoIman, 100), cuadroIman, Color.White, rotacionIman, Vector2.Zero, direccionIman, 1f);
		sprite.End();
		sprite.Begin(SpriteBlendMode.AlphaBlend, SpriteSortMode.Immediate, SaveStateMode.None, camara.transform);
	}
}
