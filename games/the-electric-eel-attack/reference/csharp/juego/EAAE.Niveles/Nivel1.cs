using Engine.Datos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE.Niveles;

internal class Nivel1 : Nivel
{
	public override void cargar(ContentManager content, GraphicsDeviceManager g)
	{
		escenario.objetos = escenario.retornarObjetos(content.RootDirectory + "\\Mapas\\nivel1");
		escenario.cargarObjetos(content);
		contarObjetos();
	}

	public override void actualizar(GameTime tiempo, Control control, Game1 game)
	{
		cortina(tiempo, game);
		if (!Escenario.Pause)
		{
			actualizarNivel(control, tiempo, game);
			moverCamara(Vector2.Zero, new Vector2(1020f, 600f));
		}
	}

	public override void dibujar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		escenario.dibujarObjetos(sprite, graphics);
	}
}
