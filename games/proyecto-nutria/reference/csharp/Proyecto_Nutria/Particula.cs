using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Particula
{
	public Vector2 posicion;

	public Vector2 direccion;

	public Vector2 origen;

	public Rectangle cuerpo;

	public Texture2D textura;

	public float zoom;

	public float rotacion;

	public float tiempo;

	public Color color;

	public bool activa;

	public void cargar(ContentManager content, string nombreTextura)
	{
		textura = content.Load<Texture2D>("Graficos/" + nombreTextura);
		origen = new Vector2(textura.Width / 2, textura.Height / 2);
	}

	public void actualizar()
	{
		actualizarCuerpo();
	}

	public void dibujar(SpriteBatch sprite)
	{
		if (activa)
		{
			sprite.Draw(textura, posicion, new Rectangle(0, 0, textura.Width, textura.Height), color, rotacion, origen, zoom, SpriteEffects.None, 1f);
		}
	}

	private void actualizarCuerpo()
	{
		cuerpo = new Rectangle((int)posicion.X, (int)posicion.Y, (int)((float)textura.Width * zoom), (int)((float)textura.Height * zoom));
	}
}
