using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria.Iconos;

internal class IconoPersonajes
{
	public enum Posicion
	{
		SuperiorIzquierda,
		SuperiorDerecha,
		InferiorIzquierda,
		InferiorDerecha
	}

	public Personaje personaje;

	private Texture2D esfera;

	private Texture2D barra;

	private Texture2D marco;

	private Texture2D moneda;

	public SpriteFont fuente;

	private Rectangle rectanguloVida;

	public void cargar(ContentManager content)
	{
		esfera = content.Load<Texture2D>("Graficos/Iconos/esfera");
		barra = content.Load<Texture2D>("Graficos/Plataformas/null");
		marco = content.Load<Texture2D>("Graficos/Iconos/marco");
		fuente = content.Load<SpriteFont>("Fuentes/fuentePredeterminada");
		moneda = content.Load<Texture2D>("Graficos/Items/moneda");
	}

	public void actualizar(Personaje p)
	{
		rectanguloVida = new Rectangle(0, 0, (int)p.vida, 20);
	}

	public void dibujar(SpriteBatch sprite, Posicion posicion)
	{
		switch (posicion)
		{
		case Posicion.SuperiorIzquierda:
			sprite.Draw(personaje.textura, new Vector2(20f, 20f), personaje.Cuadro, Color.White);
			sprite.Draw(esfera, Vector2.Zero, Color.White);
			sprite.Draw(barra, new Vector2(105f, 15f), rectanguloVida, Color.Red);
			sprite.Draw(marco, new Vector2(100f, 10f), Color.White);
			sprite.DrawString(fuente, personaje.nombre.ToString(), new Vector2(10f, 70f), Color.CornflowerBlue);
			sprite.Draw(moneda, new Vector2(150f, 40f), new Rectangle(0, 0, moneda.Width, moneda.Height), Color.Gold, 0f, Vector2.Zero, 0.3f, SpriteEffects.None, 1f);
			sprite.DrawString(fuente, personaje.monedas.ToString(), new Vector2(200f, 50f), Color.Gold);
			break;
		case Posicion.SuperiorDerecha:
			sprite.Draw(personaje.textura, new Vector2(1180f, 20f), personaje.Cuadro, Color.White);
			sprite.Draw(esfera, new Vector2(1160f, 0f), Color.White);
			sprite.Draw(barra, new Vector2(975f, 15f), rectanguloVida, Color.Red);
			sprite.Draw(marco, new Vector2(970f, 10f), Color.White);
			sprite.DrawString(fuente, personaje.nombre.ToString(), new Vector2(1180f, 70f), Color.CornflowerBlue);
			sprite.Draw(moneda, new Vector2(1000f, 40f), new Rectangle(0, 0, moneda.Width, moneda.Height), Color.Gold, 0f, Vector2.Zero, 0.3f, SpriteEffects.None, 1f);
			sprite.DrawString(fuente, personaje.monedas.ToString(), new Vector2(1050f, 50f), Color.Gold);
			break;
		}
	}
}
