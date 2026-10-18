using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal class Coipo : Personaje
{
	public Coipo()
	{
		tipo = Tipo.Coipo;
		nombre = "Coipo";
	}

	public override void cargar(ContentManager content)
	{
		textura = content.Load<Texture2D>("Graficos/Personajes/" + tipo);
		Cuadro = new Rectangle(0, 0, 70, 70);
		rotacion = 0f;
		direccion = SpriteEffects.FlipHorizontally;
		origen = new Vector2(Cuerpo.Width / 2, Cuerpo.Height / 2);
		Zoom = 1f;
		cuadros = 0f;
		Estado_ = Estado.Parado;
		VelocidadMovimiento = 0f;
		framePerSeconds = new Vector2(200f, 0f);
		maxFrameRate = textura.Width;
		polvo = new Explosion();
		polvo.cargar(content, 10, "hoja");
		vida = 200f;
		Cuerpo = new Rectangle((int)Posicion.X, (int)Posicion.Y, (int)((float)Cuadro.Width * Zoom), (int)((float)Cuadro.Height * Zoom));
		textureData = new Color[textura.Width * textura.Height];
		textura.GetData(textureData);
	}

	public override void actualizar(GameTime tiempo, Control control)
	{
		limitarMovimiento();
		correr(control);
		animar(tiempo, control);
		cambiarEstados(control);
		saltar(control);
		disparar(control, tiempo);
	}
}
