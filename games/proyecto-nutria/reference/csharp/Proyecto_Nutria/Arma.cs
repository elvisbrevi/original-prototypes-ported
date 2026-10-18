using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal abstract class Arma
{
	public Texture2D textura;

	public List<Bala> balas = new List<Bala>();

	public SpriteEffects direccion;

	public Vector2 posicion;

	public string nombre;

	public abstract void cargar(ContentManager content);

	public abstract void disparar(Bala bala, GameTime tiempo);

	public abstract void dibujar(SpriteBatch sprite, Camara camara);
}
