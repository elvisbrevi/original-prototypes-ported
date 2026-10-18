using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Objetos;

public class Particula
{
	public Texture2D Textura { get; set; }

	public Vector2 Posicion { get; set; }

	public Vector2 Direccion { get; set; }

	public float Angulo { get; set; }

	public float VelocidadAngular { get; set; }

	public Color Color_ { get; set; }

	public float Tamaño { get; set; }

	public int TiempoVida { get; set; }

	public Particula(Texture2D textura, Vector2 posicion, Vector2 direccion, float angulo, float velocidadAngular, Color color, float tamaño, int tiempoVida)
	{
		Textura = textura;
		Posicion = posicion;
		Direccion = direccion;
		Angulo = angulo;
		VelocidadAngular = velocidadAngular;
		Color_ = color;
		Tamaño = tamaño;
		TiempoVida = tiempoVida;
	}

	public void actualizar()
	{
		TiempoVida--;
		Posicion += Direccion;
		Angulo += VelocidadAngular;
	}

	public void dibujar(SpriteBatch spriteBatch)
	{
		Rectangle value = new Rectangle(0, 0, Textura.Width, Textura.Height);
		spriteBatch.Draw(origin: new Vector2(Textura.Width / 2, Textura.Height / 2), texture: Textura, position: Posicion, sourceRectangle: value, color: Color_, rotation: Angulo, scale: Tamaño, effects: SpriteEffects.None, layerDepth: 0f);
	}
}
