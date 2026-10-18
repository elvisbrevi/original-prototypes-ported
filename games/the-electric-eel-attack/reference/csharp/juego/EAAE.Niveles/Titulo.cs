using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE.Niveles;

internal class Titulo : Nivel
{
	public override void cargar(ContentManager content, GraphicsDeviceManager g)
	{
		escenario.objetos = escenario.retornarObjetos(content.RootDirectory + "\\Mapas\\titulo");
		escenario.cargarObjetos(content);
	}

	public override void actualizar(GameTime tiempo, Control control, Game1 game)
	{
		if (control.PressStart)
		{
			fin = true;
		}
		cortina(tiempo, game);
		escenario.getObjetoPorNombre("start").Fade(5, tiempo);
		escenario.getObjetoPorNombre("titulo").atenuarAditivo(1, tiempo);
		escenario.getObjetoPorNombre("titulo2").atenuarAditivo(1, tiempo);
		escenario.actualizar(tiempo);
	}

	public override void dibujar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		escenario.dibujarObjetos(sprite, graphics);
	}
}
