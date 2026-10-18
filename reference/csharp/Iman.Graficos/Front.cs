using Iman.Camaras;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Graficos;

internal class Front : Grafico
{
	public override void cargar(ContentManager content)
	{
		textura = content.Load<Texture2D>("Graficos/Fronts/" + diseño);
		y = posicion.Y;
		x = posicion.X;
		origen = new Vector2(textura.Width / 2, textura.Height / 2);
	}

	public override void actualizar(Camera2D camara, GameTime tiempo)
	{
		posicion.X = camara.position.X * -0.5f + x;
		posicion.Y = camara.position.Y * -0.5f + y;
	}

	public override void dibujar(SpriteBatch sprite, Camera2D camara)
	{
		sprite.Draw(textura, posicion, new Rectangle(0, 0, textura.Width, textura.Height), Color.White, 0f, Vector2.Zero, zoom, SpriteEffects.None, 1f);
	}
}
