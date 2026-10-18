using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Pistola : Arma
{
	public override void cargar(ContentManager content)
	{
		nombre = "Laser";
		balas = new List<Bala>(20);
		for (int i = 0; i < 20; i++)
		{
			Bala bala = new Bala();
			bala.textura = content.Load<Texture2D>("Graficos/Armas/laser");
			balas.Add(bala);
		}
		textura = content.Load<Texture2D>("Graficos/Armas/pistola");
	}

	public override void disparar(Bala bala, GameTime tiempo)
	{
		if (bala.activa)
		{
			bala.posicion += bala.direccion;
			bala.cuerpo = new Rectangle((int)bala.posicion.X, (int)bala.posicion.Y, bala.textura.Width, bala.textura.Height);
			bala.origen = new Vector2(bala.cuerpo.Width / 2, bala.cuerpo.Height / 2);
		}
	}

	public override void dibujar(SpriteBatch sprite, Camara camara)
	{
		sprite.Draw(textura, posicion, new Rectangle(0, 0, textura.Width, textura.Height), Color.White, 0f, Vector2.Zero, 0.4f, direccion, 1f);
		foreach (Bala bala in balas)
		{
			if (bala.activa)
			{
				sprite.Draw(bala.textura, new Vector2(bala.posicion.X + bala.origen.X, bala.posicion.Y + bala.origen.Y), new Rectangle(0, 0, bala.textura.Width, bala.textura.Height), Color.White, bala.rotacion, bala.origen, 1f, SpriteEffects.None, 1f);
			}
		}
	}
}
