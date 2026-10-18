using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Shaders;

public abstract class Post_Processing
{
	public float burn;

	public float saturation;

	public float r;

	public float g;

	public float b;

	public float brite;

	public float intensidad;

	public Effect efecto;

	public bool Activo = true;

	public abstract void cargar(ContentManager content);

	public abstract void actualizar(GameTime tiempo);

	public abstract void iniciar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics);

	public abstract void finalizar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics);
}
