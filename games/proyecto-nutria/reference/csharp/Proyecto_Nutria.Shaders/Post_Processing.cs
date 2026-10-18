using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria.Shaders;

internal abstract class Post_Processing
{
	public Effect efecto;

	public bool activo;

	public abstract void cargar(ContentManager content);

	public abstract void actualizar();

	public abstract void inicio(SpriteBatch sprite, Camara camara);

	public abstract void fin(SpriteBatch sprite);
}
