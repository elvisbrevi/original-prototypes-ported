using Iman.Camaras;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Graficos;

internal class Suelo : Grafico
{
	public override void cargar(ContentManager content)
	{
		textura = content.Load<Texture2D>("Graficos/Suelos/" + diseño);
		y = posicion.Y;
		x = posicion.X;
		origen = new Vector2(textura.Width / 2, textura.Height / 2);
	}

	public override void actualizar(Camera2D camara, GameTime tiempo)
	{
	}

	public override void dibujar(SpriteBatch sprite, Camera2D camara)
	{
		sprite.Draw(textura, posicion, new Rectangle(0, 0, textura.Width, textura.Height), Color.White, 0f, Vector2.Zero, zoom, SpriteEffects.None, 1f);
	}
}
