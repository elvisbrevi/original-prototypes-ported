using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Shaders;

internal abstract class Post_Processing
{
	public enum Tipo
	{
		Color_Pencil,
		Black_And_White,
		Glow
	}

	public Effect efecto;

	public bool activo;

	public bool variacion;

	public Texture2D textura;

	public float intensidad;

	public Vector2 posicion;

	public ContentManager content;

	public GraphicsDeviceManager graphics;

	public abstract void cargar(ContentManager content);

	public abstract void actualizar(GameTime tiempo);

	public abstract void iniciar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics);

	public abstract void finalizar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics);

	public abstract void iniciar(SpriteBatch spriteBatch);

	public abstract void finalizar(SpriteBatch spriteBatch);
}
