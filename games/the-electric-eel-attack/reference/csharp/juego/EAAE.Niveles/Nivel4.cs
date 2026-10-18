using System;
using Engine.Datos;
using Engine.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Audio;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE.Niveles;

internal class Nivel4 : Nivel
{
	public override void cargar(ContentManager content, GraphicsDeviceManager g)
	{
		escenario.objetos = escenario.retornarObjetos(content.RootDirectory + "\\Mapas\\nivel4");
		escenario.cargarObjetos(content);
		contarObjetos();
	}

	public override void actualizar(GameTime tiempo, Control control, Game1 game)
	{
		cortina(tiempo, game);
		moverCamara(Vector2.Zero, Vector2.Zero);
		if (Escenario.Pause)
		{
			return;
		}
		actualizarNivel(control, tiempo, game);
		if (!escenario.getObjetoPorNombre("fuego1").Activo)
		{
			for (int i = 1; i <= balas; i++)
			{
				if (escenario.detectarColision("bala" + i, "maquina"))
				{
					escenario.getObjetoPorNombre("fuego1").Activo = true;
					escenario.getObjetoPorNombre("fuego2").Activo = true;
					escenario.getObjetoPorNombre("fuego3").Activo = true;
					escenario.getObjetoPorNombre("fuego4").Activo = true;
					escenario.getObjetoPorNombre("fuego5").Activo = true;
					escenario.getObjetoPorNombre("fuego6").Activo = true;
					escenario.getObjetoPorNombre("barril1").reproducir("explosion2");
				}
			}
		}
		if (escenario.getObjetoPorNombre("fuego1").Activo && escenario.getObjetoPorNombre("barril1").Activo)
		{
			for (int i = 1; i <= 5; i++)
			{
				escenario.getObjetoPorNombre("barril" + i).Colision_ = Objeto.Colision.Ninguna;
				Random random = new Random(i);
				escenario.getObjetoPorNombre("barril" + i).Posicion.Y += 0.5f * (float)(random.NextDouble() + 0.5);
				escenario.getObjetoPorNombre("fuego" + (i + 1)).Posicion.Y = escenario.getObjetoPorNombre("barril" + i).Posicion.Y;
			}
		}
		if (escenario.getObjetoPorNombre("barril1").Activo && escenario.getObjetoPorNombre("barril1").Posicion.Y > 480f)
		{
			for (int i = 1; i <= 5; i++)
			{
				escenario.getObjetoPorNombre("barril" + i).Activo = false;
				escenario.getObjetoPorNombre("fuego" + (i + 1)).Activo = false;
			}
		}
		if (escenario.getObjetoPorNombre("hunter8").EstadoSeleccionado != 2)
		{
			escenario.getObjetoPorNombre("cañon").Inclinacion = angulo(escenario.getObjetoPorNombre("cañon").Posicion, escenario.getObjetoPorNombre("mira").Posicion);
			escenario.getObjetoPorNombre("mira").Posicion.X += (float)(0.0 - Math.Cos(angulo(escenario.getObjetoPorNombre("mira").Posicion, escenario.getObjetoPorNombre("hero").Posicion))) * 1.5f;
			escenario.getObjetoPorNombre("mira").Posicion.Y += (float)(0.0 - Math.Sin(angulo(escenario.getObjetoPorNombre("mira").Posicion, escenario.getObjetoPorNombre("hero").Posicion))) * 1.5f;
			if (escenario.detectarColision("hero", "mira") && !escenario.getObjetoPorNombre("bola").Activo)
			{
				escenario.getObjetoPorNombre("bola").reproducir("explosion1");
				escenario.getObjetoPorNombre("fuego7").Posicion = escenario.getObjetoPorNombre("bola").Posicion;
				escenario.getObjetoPorNombre("bola").Activo = true;
				escenario.getObjetoPorNombre("fuego7").Activo = true;
			}
			if (escenario.detectarColision("bola", "hero") && escenario.getObjetoPorNombre("bola").Activo)
			{
				escenario.getObjetoPorNombre("hero").reproducir("dead");
				if (escenario.getObjetoPorNombre("hero").Sonidos[3].instanciaSonido.State == SoundState.Playing)
				{
					daño = true;
				}
			}
		}
		if (escenario.getObjetoPorNombre("bola").Activo)
		{
			if (escenario.detectarColision("bola", "mira") && escenario.getObjetoPorNombre("bola").Activo)
			{
				escenario.getObjetoPorNombre("fuego7").Activo = false;
				escenario.getObjetoPorNombre("bola").Activo = false;
			}
			escenario.getObjetoPorNombre("bola").Posicion.X += (float)(0.0 - Math.Cos(angulo(escenario.getObjetoPorNombre("bola").Posicion, escenario.getObjetoPorNombre("mira").Posicion))) * 2f;
			escenario.getObjetoPorNombre("bola").Posicion.Y += (float)(0.0 - Math.Sin(angulo(escenario.getObjetoPorNombre("bola").Posicion, escenario.getObjetoPorNombre("mira").Posicion))) * 2f;
			escenario.getObjetoPorNombre("fuego7").Posicion = escenario.getObjetoPorNombre("bola").Posicion;
		}
		else
		{
			escenario.getObjetoPorNombre("bola").Posicion = new Vector2(300f, 100f);
			escenario.getObjetoPorNombre("fuego7").Posicion = new Vector2(300f, 100f);
		}
		for (int i = 1; i <= 3; i++)
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
		if (escenario.getObjetoPorNombre("hunter1").Activo && escenario.getObjetoPorNombre("hunter1").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma1").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma1").Color_ = Color.Gray;
		}
		if (escenario.getObjetoPorNombre("hunter2").Activo && escenario.getObjetoPorNombre("hunter2").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma2").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma2").Color_ = Color.Gray;
		}
		if (escenario.getObjetoPorNombre("hunter3").Activo && escenario.getObjetoPorNombre("hunter3").EstadoSeleccionado == 1)
		{
			escenario.getObjetoPorNombre("plataforma3").Color_ = Color.Yellow;
		}
		else
		{
			escenario.getObjetoPorNombre("plataforma3").Color_ = Color.Gray;
		}
	}

	public override void dibujar(SpriteBatch sprite, GraphicsDeviceManager graphics)
	{
		escenario.dibujarObjetos(sprite, graphics);
	}
}
