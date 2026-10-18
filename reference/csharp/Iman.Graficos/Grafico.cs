using Iman.Camaras;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Graficos;

internal abstract class Grafico
{
	public enum Tipo
	{
		Suelo,
		Background,
		Front,
		Cielo
	}

	public enum Diseño
	{
		_01,
		_02
	}

	public Tipo tipo;

	public Diseño diseño;

	public Texture2D textura;

	public Vector2 posicion;

	public Vector2 origen;

	public float zoom;

	public float y;

	public float x;

	public Color color = Color.White;

	public abstract void cargar(ContentManager content);

	public abstract void actualizar(Camera2D camara, GameTime tiempo);

	public abstract void dibujar(SpriteBatch sprite, Camera2D camara);
}
