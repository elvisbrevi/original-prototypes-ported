using Engine.Datos;
using Engine.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE.Niveles;

internal class Nivel2 : Nivel
{
	public override void cargar(ContentManager content, GraphicsDeviceManager g)
	{
		escenario.objetos = escenario.retornarObjetos(content.RootDirectory + "\\Mapas\\nivel2");
		escenario.cargarObjetos(content);
		contarObjetos();
	}

	public override void actualizar(GameTime tiempo, Control control, Game1 game)
	{
		cortina(tiempo, game);
		if (!Escenario.Pause)
		{
			actualizarNivel(control, tiempo, game);
			moverCamara(new Vector2(0f, -100f), new Vector2(0f, 600f));
			if (escenario.getObjetoPorNombre("hero").Posicion.X > 640f && escenario.getObjetoPorNombre("hero").Posicion.Y > 120f)
			{
				escenario.getObjetoPorNombre("hero").Posicion = new Vector2(20f, 20f);
			}
			if (escenario.getObjetoPorNombre("hero").Posicion.X < 0f && escenario.getObjetoPorNombre("hero").Posicion.Y > 120f)
			{
				escenario.getObjetoPorNombre("hero").Posicion = new Vector2(600f, 20f);
			}
			if (escenario.getObjetoPorNombre("hero").Posicion.X < 0f && escenario.getObjetoPorNombre("hero").Posicion.Y < 120f)
			{
				escenario.getObjetoPorNombre("hero").Posicion = new Vector2(600f, 160f);
			}
			if (escenario.getObjetoPorNombre("hero").Posicion.X > 640f && escenario.getObjetoPorNombre("hero").Posicion.Y < 100f)
			{
				escenario.getObjetoPorNombre("hero").Posicion = new Vector2(20f, 160f);
			}
			if (escenario.detectarColision("ladrillo1", "bloque1"))
			{
				escenario.getObjetoPorNombre("ladrillo1").Colision_ = Objeto.Colision.Total;
			}
			else
			{
				escenario.getObjetoPorNombre("ladrillo1").Colision_ = Objeto.Colision.Superior;
			}
			if (!escenario.getObjetoPorNombre("hunter1").Activo)
			{
				escenario.respawn("hunter1", new Vector2(80f, 400f), tiempo, 2f);
				escenario.getObjetoPorNombre("hunter1").EstadoSeleccionado = 0;
				escenario.getObjetoPorNombre("hunter1").Gravedad = true;
				escenario.getObjetoPorNombre("hunter1").Inclinacion = 0f;
				escenario.getObjetoPorNombre("hunter1").Lugar_ = Objeto.Lugar.Medio;
			}
		}
	}

	public override void dibujar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		escenario.dibujarObjetos(sprite, graphics);
	}
}
