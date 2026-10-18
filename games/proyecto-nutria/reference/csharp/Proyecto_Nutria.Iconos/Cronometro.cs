using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria.Iconos;

internal class Cronometro
{
	private double segundos;

	private double minutos;

	private double velocidad;

	private SpriteFont fuente;

	private string txtSegundos;

	private string txtMinutos;

	private Color color = Color.CornflowerBlue;

	public void cargar(ContentManager content)
	{
		fuente = content.Load<SpriteFont>("Fuentes/fuenteCronometro");
		minutos = 5.0;
		segundos = 0.0;
		velocidad = 2.0;
	}

	public void actualizar(GameTime tiempo)
	{
		segundos -= tiempo.ElapsedGameTime.TotalSeconds * velocidad;
		if (segundos < 0.0 && minutos > 0.0)
		{
			segundos = 60.0;
			minutos -= 1.0;
		}
		txtSegundos = segundos.ToString();
		txtMinutos = minutos.ToString();
	}

	public void dibujar(SpriteBatch sprite)
	{
		if (minutos < 5.0)
		{
			sprite.DrawString(fuente, "0", new Vector2(500f, -40f), color);
		}
		sprite.DrawString(fuente, txtMinutos, new Vector2(570f, -40f), color);
		sprite.DrawString(fuente, ":", new Vector2(640f, -40f), color);
		if (segundos > 9.899999618530273)
		{
			sprite.DrawString(fuente, segundos.ToString("0"), new Vector2(680f, -40f), color);
			return;
		}
		sprite.DrawString(fuente, "0", new Vector2(680f, -40f), color);
		sprite.DrawString(fuente, txtSegundos.Remove(1), new Vector2(750f, -40f), color);
	}
}
