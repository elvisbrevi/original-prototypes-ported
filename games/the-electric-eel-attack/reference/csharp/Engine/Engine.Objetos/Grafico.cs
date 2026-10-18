using Engine.Datos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Objetos;

public class Grafico : Objeto
{
	private float x;

	private float y;

	public override void cargar(ContentManager content)
	{
		Textura = content.Load<Texture2D>("Graficos/" + NombreTextura);
		Origen = new Vector2((float)Textura.Width / (2f * Tamaño), (float)Textura.Height / (2f * Tamaño));
		TextureData = new Color[Textura.Width * Textura.Height];
	}

	public override void actualizar(GameTime tiempo)
	{
		actualizarCuerpo();
		scroll();
		if (Shader_ == Shader.MonoSketch || Shader_ == Shader.ColorSketch)
		{
			PostProcessing.actualizar(tiempo);
		}
		if (Movimento)
		{
			mover();
		}
		if (Animado && Estados.Count > 0)
		{
			Estados[EstadoSeleccionado].animar(tiempo, this);
		}
	}

	private void scroll()
	{
		x = Posicion.X + (0f - Pantalla.Posicion.X) * ScrollHorizontal;
		y = Posicion.Y + (0f - Pantalla.Posicion.Y) * ScrollVertical;
		if (!Movimento)
		{
			if (repetirScroll && (float)Cuerpo.Right < Pantalla.Posicion.X)
			{
				Posicion.X = Pantalla.Posicion.X + (float)Pantalla._VWidth;
			}
			if (repetirScroll && (float)Cuerpo.Left > Pantalla.Posicion.X + (float)Pantalla._VWidth)
			{
				Posicion.X = Pantalla.Posicion.X - (float)Cuerpo.Width;
			}
			if (repetirScroll && (float)Cuerpo.Bottom < Pantalla.Posicion.Y)
			{
				Posicion.Y = Pantalla.Posicion.Y + (float)Pantalla._VHeight;
			}
			if (repetirScroll && (float)Cuerpo.Top > Pantalla.Posicion.Y + (float)Pantalla._VHeight)
			{
				Posicion.Y = Pantalla.Posicion.Y - (float)Cuerpo.Height;
			}
		}
		else
		{
			if (repetirScroll && (float)Cuerpo.Left >= Destino.X && Destino.X != Inicio.X)
			{
				Posicion.X = Inicio.X - (float)Cuerpo.Width;
			}
			if (repetirScroll && (float)Cuerpo.Top >= Destino.Y && Destino.Y != Inicio.Y)
			{
				Posicion.Y = Inicio.Y - (float)Cuerpo.Height;
			}
		}
	}

	private void actualizarCuerpo()
	{
		if (Estados.Count > 0)
		{
			Cuerpo = new Rectangle((int)x, (int)y, (int)((float)Estados[EstadoSeleccionado].Cuadro.Width * Tamaño), (int)((float)Estados[EstadoSeleccionado].Cuadro.Height * Tamaño));
		}
		else
		{
			Cuerpo = new Rectangle((int)x, (int)y, (int)((float)Textura.Width * Tamaño), (int)((float)Textura.Height * Tamaño));
		}
		if (Estados.Count > 0)
		{
			Origen = new Vector2(Estados[EstadoSeleccionado].Cuadro.Width / 2, Estados[EstadoSeleccionado].Cuadro.Height / 2);
		}
		else
		{
			Origen = new Vector2(Textura.Width / 2, Textura.Height / 2);
		}
	}

	public override void dibujar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		if (fueraDeCamara)
		{
			spriteBatch.Begin(blendMode, SpriteSortMode.Immediate, SaveStateMode.None);
		}
		else
		{
			spriteBatch.Begin(blendMode, SpriteSortMode.Immediate, SaveStateMode.None, Pantalla.getTransformationMatrix());
		}
		if (Escenario.ActivarShader && Escenario.Shader_ != Shader.Ninguno && !noShaderCamara)
		{
			Escenario.PostProcessing.iniciar(spriteBatch, graphics);
		}
		else if (Shader_ != Shader.Ninguno && !Escenario.ActivarShader)
		{
			PostProcessing.iniciar(spriteBatch, graphics);
		}
		if (Estados.Count == 0)
		{
			spriteBatch.Draw(Textura, new Vector2(x + Origen.X * Tamaño, y + Origen.Y * Tamaño), new Rectangle(0, 0, Textura.Width, Textura.Height), Color_, Inclinacion, Origen, Tamaño, SpriteEffect_, 1f);
		}
		else if (Estados.Count > 0)
		{
			spriteBatch.Draw(Textura, new Vector2(x + Origen.X * Tamaño, y + Origen.Y * Tamaño), Estados[EstadoSeleccionado].Cuadro, Color_, Inclinacion, Origen, Tamaño, SpriteEffect_, 1f);
		}
		if (Escenario.ActivarShader && Escenario.Shader_ != Shader.Ninguno && !noShaderCamara)
		{
			Escenario.PostProcessing.finalizar(spriteBatch, graphics);
		}
		else if (Shader_ != Shader.Ninguno && !Escenario.ActivarShader)
		{
			PostProcessing.finalizar(spriteBatch, graphics);
		}
		if (Escenario.MostrarCuerpos)
		{
			spriteBatch.Draw(Escenario.pixel, new Vector2(x + Origen.X * Tamaño, y + Origen.Y * Tamaño), new Rectangle(Cuerpo.X, Cuerpo.Y, (int)((float)Cuerpo.Width / Tamaño), (int)((float)Cuerpo.Height / Tamaño)), new Color(Color.Blue, 100), 0f, Origen, Tamaño, SpriteEffect_, 1f);
		}
		spriteBatch.End();
	}
}
