using Engine.Datos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Objetos;

public class Texto : Objeto
{
	public override void cargar(ContentManager content)
	{
		Fuente = content.Load<SpriteFont>("Fuentes/" + NombreTextura);
	}

	public override void actualizar(GameTime tiempo)
	{
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
		spriteBatch.DrawString(Fuente, Texto, Posicion, Color_, Inclinacion, Vector2.Zero, Tamaño, SpriteEffect_, 1f);
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
			spriteBatch.Draw(Escenario.pixel, Cuerpo, new Color(Color.Blue, 100));
		}
		spriteBatch.End();
	}
}
