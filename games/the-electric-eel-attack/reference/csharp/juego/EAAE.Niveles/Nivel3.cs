using Engine.Datos;
using Engine.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Audio;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE.Niveles;

internal class Nivel3 : Nivel
{
	public override void cargar(ContentManager content, GraphicsDeviceManager g)
	{
		escenario.objetos = escenario.retornarObjetos(content.RootDirectory + "\\Mapas\\nivel3");
		escenario.cargarObjetos(content);
		contarObjetos();
	}

	public override void actualizar(GameTime tiempo, Control control, Game1 game)
	{
		cortina(tiempo, game);
		if (Escenario.Pause)
		{
			return;
		}
		actualizarNivel(control, tiempo, game);
		moverCamara(new Vector2(0f, -150f), new Vector2(1020f, 600f));
		for (int i = 1; i <= 4; i++)
		{
			if (escenario.detectarColision("plataforma" + i, "hero") && escenario.getObjetoPorNombre("plataforma" + i).Color_ == Color.Yellow)
			{
				escenario.getObjetoPorNombre("hero").reproducir("dead");
				if (escenario.getObjetoPorNombre("hero").Sonidos[3].instanciaSonido.State == SoundState.Playing)
				{
					daño = true;
				}
			}
		}
		if (escenario.getObjetoPorNombre("hunter2").Activo && escenario.getObjetoPorNombre("hunter2").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma1").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma1").Color_ = Color.Gray;
		}
		if (escenario.getObjetoPorNombre("hunter3").Activo && escenario.getObjetoPorNombre("hunter3").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma2").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma2").Color_ = Color.Gray;
		}
		if (escenario.getObjetoPorNombre("hunter4").Activo && escenario.getObjetoPorNombre("hunter4").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma3").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma3").Color_ = Color.Gray;
		}
		if (escenario.getObjetoPorNombre("hunter5").Activo && escenario.getObjetoPorNombre("hunter5").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma4").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma4").Color_ = Color.Gray;
		}
		if ((escenario.getObjetoPorNombre("hunter8").Posicion.X < 790f && escenario.getObjetoPorNombre("hunter8").Posicion.Y > 539f) || !escenario.getObjetoPorNombre("hunter8").Activo)
		{
			escenario.getObjetoPorNombre("hunter8").Activo = false;
			escenario.getObjetoPorNombre("hunter8").EstadoSeleccionado = 0;
			escenario.getObjetoPorNombre("hunter8").Inclinacion = 0f;
			escenario.getObjetoPorNombre("hunter8").Lugar_ = Objeto.Lugar.Medio;
			escenario.getObjetoPorNombre("hunter8").Gravedad = true;
			escenario.respawn("hunter8", new Vector2(940f, 0f), tiempo, 5f);
		}
		if ((escenario.getObjetoPorNombre("hunter7").Posicion.X > 790f && escenario.getObjetoPorNombre("hunter7").Posicion.Y > 539f) || !escenario.getObjetoPorNombre("hunter7").Activo)
		{
			escenario.getObjetoPorNombre("hunter7").Activo = false;
			escenario.getObjetoPorNombre("hunter7").EstadoSeleccionado = 0;
			escenario.getObjetoPorNombre("hunter7").Inclinacion = 0f;
			escenario.getObjetoPorNombre("hunter7").Lugar_ = Objeto.Lugar.Medio;
			escenario.getObjetoPorNombre("hunter7").Gravedad = true;
			escenario.respawn("hunter7", new Vector2(940f, 0f), tiempo, 5f);
		}
		if ((escenario.getObjetoPorNombre("hunter6").Posicion.X > 698f && escenario.getObjetoPorNombre("hunter6").Posicion.Y > 539f) || !escenario.getObjetoPorNombre("hunter6").Activo)
		{
			escenario.getObjetoPorNombre("hunter6").Activo = false;
			escenario.getObjetoPorNombre("hunter6").EstadoSeleccionado = 0;
			escenario.getObjetoPorNombre("hunter6").Inclinacion = 0f;
			escenario.getObjetoPorNombre("hunter6").Lugar_ = Objeto.Lugar.Medio;
			escenario.getObjetoPorNombre("hunter6").Gravedad = true;
			escenario.respawn("hunter6", new Vector2(540f, 0f), tiempo, 5f);
		}
		if ((escenario.getObjetoPorNombre("hunter9").Posicion.X > 698f && escenario.getObjetoPorNombre("hunter9").Posicion.Y > 539f) || !escenario.getObjetoPorNombre("hunter9").Activo)
		{
			escenario.getObjetoPorNombre("hunter9").Activo = false;
			escenario.getObjetoPorNombre("hunter9").EstadoSeleccionado = 0;
			escenario.getObjetoPorNombre("hunter9").Inclinacion = 0f;
			escenario.getObjetoPorNombre("hunter9").Lugar_ = Objeto.Lugar.Medio;
			escenario.getObjetoPorNombre("hunter9").Gravedad = true;
			escenario.respawn("hunter9", new Vector2(540f, 0f), tiempo, 5f);
		}
	}

	public override void dibujar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		escenario.dibujarObjetos(sprite, graphics);
	}
}
