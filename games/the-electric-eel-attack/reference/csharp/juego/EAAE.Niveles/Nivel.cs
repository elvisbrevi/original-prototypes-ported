using System;
using Engine.Datos;
using Engine.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Audio;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace EAAE.Niveles;

public abstract class Nivel
{
	protected bool fin;

	protected bool saltando;

	protected bool daño;

	protected int distancia;

	protected int mapas;

	protected int hunters;

	protected int kamikazes;

	protected int pajaros;

	protected int bloques;

	protected int bateria;

	protected int pilas;

	protected int contadorMapas;

	protected int balas;

	protected int fachadas;

	protected int paredes;

	protected int huevos;

	public Escenario escenario = new Escenario();

	public Escenario Escenario_
	{
		get
		{
			return escenario;
		}
		set
		{
			escenario = value;
		}
	}

	public bool Fin
	{
		get
		{
			return fin;
		}
		set
		{
			fin = value;
		}
	}

	public abstract void cargar(ContentManager content, GraphicsDeviceManager g);

	public abstract void actualizar(GameTime tiempo, Control control, Game1 game);

	public abstract void dibujar(SpriteBatch sprite, GraphicsDeviceManager graphics);

	protected float angulo(Vector2 a, Vector2 b)
	{
		return (float)Math.Atan2(a.Y - b.Y, a.X - b.X);
	}

	protected void actualizarNivel(Control control, GameTime tiempo, Game1 game)
	{
		if (game.nivelActivo != 0 && control.Reiniciar)
		{
			game.cambiarNivel(game.nivelActivo);
		}
		if (escenario.getObjetoPorNombre("hero").Activo)
		{
			moverPersonaje(control, tiempo, game);
		}
		actualizarObjetos(tiempo, game);
		escenario.actualizar(tiempo);
	}

	protected void cortina(GameTime tiempo, Game1 game)
	{
		if (!escenario.getObjetoPorNombre("pixel").Activo && escenario.getObjetoPorNombre("pixel").Color_.A > 250)
		{
			escenario.getObjetoPorNombre("pixel").Activo = true;
		}
		if (escenario.getObjetoPorNombre("pixel").Activo && !daño && !fin)
		{
			escenario.getObjetoPorNombre("pixel").FadeOut(3, tiempo);
		}
		if (!escenario.getObjetoPorNombre("pixel").Activo && (daño || fin))
		{
			escenario.getObjetoPorNombre("pixel").Activo = true;
		}
		if (escenario.getObjetoPorNombre("pixel").Activo && (daño || fin))
		{
			escenario.getObjetoPorNombre("pixel").FadeIn(3, tiempo);
		}
		if (daño)
		{
			Escenario.Pause = true;
			escenario.activarShaders(Objeto.Shader.Negative, game.Content);
		}
		else if (!fin)
		{
			escenario.desactivarShaders();
		}
		if (fin && game.nivelActivo > 0)
		{
			escenario.getObjetoPorNombre("hero").Activo = false;
		}
		if (fin && escenario.getObjetoPorNombre("pixel").Color_.A == byte.MaxValue)
		{
			switch (game.nivelActivo)
			{
			case 0:
				game.cambiarNivel(1);
				fin = false;
				Escenario.Pause = false;
				break;
			case 1:
				game.cambiarNivel(2);
				fin = false;
				Escenario.Pause = false;
				break;
			case 2:
				game.cambiarNivel(3);
				fin = false;
				Escenario.Pause = false;
				break;
			case 3:
				game.cambiarNivel(4);
				fin = false;
				Escenario.Pause = false;
				break;
			case 4:
				game.cambiarNivel(0);
				fin = false;
				Escenario.Pause = false;
				break;
			}
		}
		if (daño && escenario.getObjetoPorNombre("pixel").Color_.A == byte.MaxValue)
		{
			game.cambiarNivel(game.nivelActivo);
			daño = false;
			Escenario.Pause = false;
		}
	}

	protected void contarObjetos()
	{
		contadorMapas = 0;
		mapas = 0;
		hunters = 0;
		kamikazes = 0;
		pajaros = 0;
		bloques = 0;
		bateria = 0;
		pilas = 0;
		balas = 0;
		fachadas = 0;
		paredes = 0;
		paredes = 0;
		huevos = 0;
		saltando = false;
		foreach (Objeto objeto in escenario.objetos)
		{
			if (objeto.Nombre.Contains("huevo"))
			{
				huevos++;
			}
		}
		foreach (Objeto objeto2 in escenario.objetos)
		{
			if (objeto2.Nombre.Contains("pared"))
			{
				paredes++;
			}
		}
		foreach (Objeto objeto3 in escenario.objetos)
		{
			if (objeto3.Nombre.Contains("fachada"))
			{
				fachadas++;
			}
		}
		foreach (Objeto objeto4 in escenario.objetos)
		{
			if (objeto4.Nombre.Contains("bala"))
			{
				balas++;
			}
		}
		foreach (Objeto objeto5 in escenario.objetos)
		{
			if (objeto5.Nombre.Contains("pila"))
			{
				pilas++;
			}
		}
		foreach (Objeto objeto6 in escenario.objetos)
		{
			if (objeto6.Nombre.Contains("bloque"))
			{
				bloques++;
			}
		}
		foreach (Objeto objeto7 in escenario.objetos)
		{
			if (objeto7.Nombre.Contains("mapa"))
			{
				mapas++;
				contadorMapas++;
			}
		}
		foreach (Objeto objeto8 in escenario.objetos)
		{
			if (objeto8.Nombre.Contains("hunter"))
			{
				hunters++;
			}
		}
		foreach (Objeto objeto9 in escenario.objetos)
		{
			if (objeto9.Nombre.Contains("kamikaze"))
			{
				kamikazes++;
			}
		}
		foreach (Objeto objeto10 in escenario.objetos)
		{
			if (objeto10.Nombre.Contains("pajaro"))
			{
				pajaros++;
			}
		}
		escenario.getObjetoPorNombre("barra").Estados[0].Cuadro.Width = bateria;
	}

	protected void moverCamara(Vector2 min, Vector2 max)
	{
		Pantalla.Posicion.X = escenario.getObjetoPorNombre("hero").Posicion.X + (float)(escenario.getObjetoPorNombre("hero").Cuerpo.Width / 2) - (float)(Pantalla._Width / 2);
		Pantalla.Posicion.X = MathHelper.Clamp(Pantalla.Posicion.X, min.X, max.X - (float)Pantalla._Width);
		Pantalla.Posicion.Y = escenario.getObjetoPorNombre("hero").Posicion.Y + (float)(escenario.getObjetoPorNombre("hero").Cuerpo.Height / 2) - (float)(Pantalla._Height / 2);
		Pantalla.Posicion.Y = MathHelper.Clamp(Pantalla.Posicion.Y, min.Y, max.Y - (float)Pantalla._Height);
		Pantalla.reset();
	}

	private void actualizarObjetos(GameTime tiempo, Game1 game)
	{
		for (int i = 1; i <= fachadas; i++)
		{
			if (escenario.detectarColision("hero", "fachada" + i))
			{
				escenario.getObjetoPorNombre("fachada" + i).FadeOut(4, tiempo);
			}
			else
			{
				escenario.getObjetoPorNombre("fachada" + i).FadeIn(4, tiempo);
			}
		}
		for (int i = 1; i <= balas; i++)
		{
			if (!escenario.getObjetoPorNombre("bala" + i).Activo)
			{
				continue;
			}
			escenario.getObjetoPorNombre("bala" + i).Velocidad += (float)tiempo.ElapsedGameTime.TotalSeconds * 2f;
			if (escenario.getObjetoPorNombre("bala" + i).Velocidad > 10f)
			{
				escenario.getObjetoPorNombre("bala" + i).Activo = false;
			}
			for (int j = 1; j <= huevos; j++)
			{
				if (escenario.detectarColision("bala" + i, "huevo" + j) && escenario.getObjetoPorNombre("huevo" + j).Activo && escenario.getObjetoPorNombre("huevo" + j).EstadoSeleccionado == 0)
				{
					escenario.getObjetoPorNombre("bala" + i).Activo = false;
					escenario.getObjetoPorNombre("huevo" + j).cambiarEstado(1);
				}
			}
			for (int j = 1; j <= hunters; j++)
			{
				if (escenario.detectarColision("bala" + i, "hunter" + j) && escenario.getObjetoPorNombre("hunter" + j).Activo)
				{
					escenario.getObjetoPorNombre("bala" + i).Activo = false;
					escenario.getObjetoPorNombre("hunter" + j).cambiarEstado(2);
					escenario.getObjetoPorNombre("hunter" + j).reproducir("hunter dead");
				}
			}
			for (int j = 1; j <= paredes; j++)
			{
				if (escenario.detectarColision("bala" + i, "pared" + j) && escenario.getObjetoPorNombre("pared" + j).Activo)
				{
					escenario.getObjetoPorNombre("bala" + i).Activo = false;
				}
			}
			for (int j = 1; j <= kamikazes; j++)
			{
				if (escenario.detectarColision("bala" + i, "kamikaze" + j) && escenario.getObjetoPorNombre("kamikaze" + j).Activo)
				{
					escenario.getObjetoPorNombre("bala" + i).Activo = false;
					escenario.getObjetoPorNombre("kamikaze" + j).cambiarEstado(2);
					escenario.getObjetoPorNombre("kamikaze" + j).reproducir("kamikaze dead");
				}
			}
			switch (escenario.getObjetoPorNombre("bala" + i).SpriteEffect_)
			{
			case SpriteEffects.None:
				escenario.getObjetoPorNombre("bala" + i).Posicion.X += escenario.getObjetoPorNombre("bala" + i).Velocidad;
				break;
			case SpriteEffects.FlipHorizontally:
				escenario.getObjetoPorNombre("bala" + i).Posicion.X -= escenario.getObjetoPorNombre("bala" + i).Velocidad;
				break;
			}
		}
		if (escenario.detectarColision("hero", "vortice") && escenario.getObjetoPorNombre("vortice").Activo)
		{
			fin = true;
		}
		if (contadorMapas == 0)
		{
			for (int i = 1; i <= hunters; i++)
			{
				escenario.getObjetoPorNombre("hunter" + i).Activo = false;
			}
			for (int i = 1; i <= kamikazes; i++)
			{
				escenario.getObjetoPorNombre("kamikaze" + i).Activo = false;
			}
			if (!escenario.getObjetoPorNombre("vortice").Activo)
			{
				escenario.getObjetoPorNombre("vortice").reproducir("vortice");
			}
			escenario.getObjetoPorNombre("vortice").Activo = true;
			escenario.activarShaders(Objeto.Shader.Brightness, game.Content);
			Escenario.PostProcessing.intensidad = 2f;
		}
		else if (!daño)
		{
			escenario.desactivarShaders();
		}
		for (int i = 1; i <= mapas; i++)
		{
			if (escenario.detectarColision("hero", "mapa" + i) && escenario.getObjetoPorNombre("mapa" + i).Activo && !escenario.getObjetoPorNombre("mapa" + i).fueraDeCamara)
			{
				escenario.getObjetoPorNombre("mapa" + i).reproducir("paper");
				escenario.getObjetoPorNombre("mapa" + i).Posicion = new Vector2(80 + 40 * i, 10f);
				escenario.getObjetoPorNombre("mapa" + i).fueraDeCamara = true;
				escenario.getObjetoPorNombre("mapa" + i).noShaderCamara = true;
				escenario.getObjetoPorNombre("mapa" + i).Lugar_ = Objeto.Lugar.Frente;
				contadorMapas--;
			}
		}
		for (int i = 1; i <= pilas; i++)
		{
			if (escenario.detectarColision("hero", "pila" + i) && escenario.getObjetoPorNombre("pila" + i).Activo)
			{
				bateria = 100;
				escenario.getObjetoPorNombre("barra").Estados[0].Cuadro.Width = bateria;
				escenario.getObjetoPorNombre("pila" + i).reproducir("charge");
				if (escenario.getObjetoPorNombre("pila" + i).Sonidos[0].instanciaSonido.State == SoundState.Playing)
				{
					escenario.getObjetoPorNombre("pila" + i).Activo = false;
				}
			}
		}
		for (int i = 1; i <= bloques; i++)
		{
			if (escenario.getObjetoPorNombre("bloque" + i).Activo && escenario.detectarColision("hero", "bloque" + i))
			{
				if (escenario.getObjetoPorNombre("hero").colisionIzquierda && !escenario.getObjetoPorNombre("bloque" + 1).colisionIzquierda)
				{
					escenario.getObjetoPorNombre("bloque" + i).Posicion.X -= escenario.getObjetoPorNombre("hero").Velocidad / 2f;
				}
				if (escenario.getObjetoPorNombre("hero").colisionDerecha && !escenario.getObjetoPorNombre("bloque" + 1).colisionDerecha)
				{
					escenario.getObjetoPorNombre("bloque" + i).Posicion.X += escenario.getObjetoPorNombre("hero").Velocidad / 2f;
				}
			}
		}
		for (int i = 1; i <= pajaros; i++)
		{
			if (escenario.getObjetoPorNombre("pajaro" + i).Activo)
			{
				if (escenario.getObjetoPorNombre("pajaro" + i).Cuerpo.Intersects(escenario.getObjetoPorNombre("hero").Cuerpo))
				{
					escenario.getObjetoPorNombre("pajaro" + i).EstadoSeleccionado = 1;
					escenario.getObjetoPorNombre("pajaro" + i).Movimento = true;
					escenario.getObjetoPorNombre("pajaro" + i).reproducir("crow", escenario.getObjetoPorNombre("pajaro" + i).Posicion, escenario.getObjetoPorNombre("hero").Posicion);
				}
				if (escenario.getObjetoPorNombre("pajaro" + i).Posicion.X > 1024f || escenario.getObjetoPorNombre("pajaro" + i).Cuerpo.Right < 0)
				{
					escenario.getObjetoPorNombre("pajaro" + i).Activo = false;
				}
			}
		}
		for (int i = 1; i <= hunters; i++)
		{
			if (escenario.getObjetoPorNombre("hunter" + i).EstadoSeleccionado == 2)
			{
				if (escenario.getObjetoPorNombre("hunter" + i).Posicion.Y <= (float)Pantalla._VHeight + Pantalla.Posicion.Y)
				{
					escenario.getObjetoPorNombre("hunter" + i).Lugar_ = Objeto.Lugar.Frente;
					escenario.getObjetoPorNombre("hunter" + i).Gravedad = false;
					escenario.getObjetoPorNombre("hunter" + i).Posicion.Y += 4f;
				}
				else
				{
					escenario.getObjetoPorNombre("hunter" + i).Activo = false;
				}
				continue;
			}
			escenario.getObjetoPorNombre("hunter" + i).Color_.B -= (byte)(10f * ((float)tiempo.ElapsedGameTime.TotalSeconds * 10f));
			if (escenario.getObjetoPorNombre("hunter" + i).Color_.B > 100 && escenario.getObjetoPorNombre("hunter" + i).Color_.B < 150)
			{
				escenario.getObjetoPorNombre("hunter" + i).EstadoSeleccionado = 1;
			}
			else
			{
				escenario.getObjetoPorNombre("hunter" + i).EstadoSeleccionado = 0;
			}
			if (escenario.detectarColision("hero", "hunter" + i))
			{
				if (pisar(escenario.getObjetoPorNombre("hunter" + i)) && escenario.getObjetoPorNombre("hunter" + i).EstadoSeleccionado == 0 && escenario.getObjetoPorNombre("hunter" + i).Activo)
				{
					escenario.getObjetoPorNombre("hunter" + i).cambiarEstado(2);
					escenario.getObjetoPorNombre("hunter" + i).reproducir("hunter dead");
				}
				else if (escenario.getObjetoPorNombre("hunter" + i).Activo)
				{
					daño = true;
					escenario.getObjetoPorNombre("hero").reproducir("dead");
				}
			}
			if (escenario.getObjetoPorNombre("hunter" + i).idaX)
			{
				escenario.getObjetoPorNombre("hunter" + i).SpriteEffect_ = SpriteEffects.FlipHorizontally;
			}
			else
			{
				escenario.getObjetoPorNombre("hunter" + i).SpriteEffect_ = SpriteEffects.None;
			}
			if (escenario.getObjetoPorNombre("hunter" + i).Movimento && escenario.getObjetoPorNombre("hunter" + i).Activo && escenario.getObjetoPorNombre("hunter" + i).EstadoSeleccionado == 0)
			{
				escenario.getObjetoPorNombre("hunter" + i).reproducir("hunter move", escenario.getObjetoPorNombre("hunter" + i).Posicion, escenario.getObjetoPorNombre("hero").Posicion);
			}
			else if (escenario.getObjetoPorNombre("hunter" + i).Activo && escenario.getObjetoPorNombre("hunter" + i).EstadoSeleccionado == 1)
			{
				escenario.getObjetoPorNombre("hunter" + i).reproducir("electricity", escenario.getObjetoPorNombre("hunter" + i).Posicion, escenario.getObjetoPorNombre("hero").Posicion);
			}
		}
		for (int i = 1; i <= kamikazes; i++)
		{
			if (escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado == 2)
			{
				if (escenario.getObjetoPorNombre("kamikaze" + i).Posicion.Y <= (float)Pantalla._VHeight + Pantalla.Posicion.Y)
				{
					escenario.getObjetoPorNombre("kamikaze" + i).Lugar_ = Objeto.Lugar.Frente;
					escenario.getObjetoPorNombre("kamikaze" + i).Gravedad = false;
					escenario.getObjetoPorNombre("kamikaze" + i).Posicion.Y += 4f;
				}
				else
				{
					escenario.getObjetoPorNombre("kamikaze" + i).Activo = false;
				}
				continue;
			}
			if (escenario.detectarColision("hero", "kamikaze" + i))
			{
				if (pisar(escenario.getObjetoPorNombre("kamikaze" + i)) && escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado == 0 && escenario.getObjetoPorNombre("kamikaze" + i).Activo)
				{
					escenario.getObjetoPorNombre("kamikaze" + i).cambiarEstado(2);
					escenario.getObjetoPorNombre("kamikaze" + i).reproducir("kamikaze dead");
				}
				else if (escenario.getObjetoPorNombre("kamikaze" + i).Activo && !daño)
				{
					daño = true;
					escenario.getObjetoPorNombre("hero").reproducir("dead");
				}
			}
			if (escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado == 0)
			{
				escenario.getObjetoPorNombre("kamikaze" + i).reproducir("kamikaze move", escenario.getObjetoPorNombre("kamikaze" + i).Posicion, escenario.getObjetoPorNombre("hero").Posicion);
			}
			else if (escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado == 1)
			{
				escenario.getObjetoPorNombre("kamikaze" + i).reproducir("electricity", escenario.getObjetoPorNombre("kamikaze" + i).Posicion, escenario.getObjetoPorNombre("hero").Posicion);
			}
			if (escenario.getObjetoPorNombre("kamikaze" + i).idaY && escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado != 2)
			{
				escenario.getObjetoPorNombre("kamikaze" + i).SpriteEffect_ = SpriteEffects.FlipVertically;
				escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado = 0;
			}
			else if (escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado != 2)
			{
				escenario.getObjetoPorNombre("kamikaze" + i).SpriteEffect_ = SpriteEffects.None;
				escenario.getObjetoPorNombre("kamikaze" + i).EstadoSeleccionado = 1;
			}
		}
	}

	private bool pisar(Objeto enemigo)
	{
		if (escenario.getObjetoPorNombre("hero").Cuerpo.Bottom >= enemigo.Cuerpo.Top && escenario.getObjetoPorNombre("hero").Cuerpo.Bottom <= enemigo.Cuerpo.Top + 10 && !saltando && enemigo.Activo)
		{
			distancia = (int)(escenario.getObjetoPorNombre("hero").Posicion.Y - 70f);
			escenario.getObjetoPorNombre("hero").FuerzaGravedad = -8f;
			saltando = true;
			return true;
		}
		return false;
	}

	private void moverPersonaje(Control control, GameTime tiempo, Game1 game)
	{
		if (control.PressZ && bateria > 0)
		{
			escenario.getObjetoPorNombre("hero").reproducir("shot");
			escenario.getObjetoPorNombre("bala" + bateria / 10).SpriteEffect_ = escenario.getObjetoPorNombre("hero").SpriteEffect_;
			switch (escenario.getObjetoPorNombre("hero").SpriteEffect_)
			{
			case SpriteEffects.None:
				escenario.getObjetoPorNombre("bala" + bateria / 10).Posicion = new Vector2(escenario.getObjetoPorNombre("hero").Cuerpo.Right, escenario.getObjetoPorNombre("hero").Posicion.Y + 17f);
				break;
			case SpriteEffects.FlipHorizontally:
				escenario.getObjetoPorNombre("bala" + bateria / 10).Posicion = new Vector2(escenario.getObjetoPorNombre("hero").Cuerpo.Left, escenario.getObjetoPorNombre("hero").Posicion.Y + 17f);
				break;
			}
			escenario.getObjetoPorNombre("bala" + bateria / 10).Velocidad = 9f;
			escenario.getObjetoPorNombre("bala" + bateria / 10).Activo = true;
			bateria -= 10;
			escenario.getObjetoPorNombre("barra").Estados[0].Cuadro.Width = bateria;
		}
		if (control.HoldPressRight)
		{
			escenario.getObjetoPorNombre("hero").Velocidad = 4f;
			escenario.getObjetoPorNombre("hero").Posicion.X += escenario.getObjetoPorNombre("hero").Velocidad;
			if (!saltando && escenario.getObjetoPorNombre("hero").EstadoSeleccionado != 1)
			{
				escenario.getObjetoPorNombre("hero").cambiarEstado(1);
			}
			escenario.getObjetoPorNombre("hero").SpriteEffect_ = SpriteEffects.None;
			escenario.getObjetoPorNombre("hero").reproducir("walk");
		}
		if (control.HoldPressLeft)
		{
			escenario.getObjetoPorNombre("hero").Velocidad = 4f;
			escenario.getObjetoPorNombre("hero").Posicion.X -= escenario.getObjetoPorNombre("hero").Velocidad;
			if (!saltando && escenario.getObjetoPorNombre("hero").EstadoSeleccionado != 1)
			{
				escenario.getObjetoPorNombre("hero").cambiarEstado(1);
			}
			escenario.getObjetoPorNombre("hero").SpriteEffect_ = SpriteEffects.FlipHorizontally;
			escenario.getObjetoPorNombre("hero").reproducir("walk");
		}
		if (!control.HoldPressLeft && !control.HoldPressRight)
		{
			if (!saltando && escenario.getObjetoPorNombre("hero").EstadoSeleccionado != 0)
			{
				escenario.getObjetoPorNombre("hero").cambiarEstado(0);
			}
			escenario.getObjetoPorNombre("hero").stop("walk");
		}
		if (!saltando && control.PressX && escenario.getObjetoPorNombre("hero").colisionSuperior)
		{
			distancia = (int)(escenario.getObjetoPorNombre("hero").Posicion.Y - 50f);
			escenario.getObjetoPorNombre("hero").FuerzaGravedad = -6f;
			saltando = true;
		}
		if (saltando)
		{
			escenario.getObjetoPorNombre("hero").reproducir("jump");
			if (escenario.getObjetoPorNombre("hero").Posicion.Y > (float)distancia)
			{
				escenario.getObjetoPorNombre("hero").Posicion.Y -= 1.9f;
				if (escenario.getObjetoPorNombre("hero").EstadoSeleccionado != 2)
				{
					escenario.getObjetoPorNombre("hero").cambiarEstado(2);
				}
			}
			else
			{
				saltando = false;
			}
		}
		if (!escenario.getObjetoPorNombre("hero").colisionSuperior)
		{
			escenario.getObjetoPorNombre("hero").stop("walk");
			if (escenario.getObjetoPorNombre("hero").EstadoSeleccionado != 2)
			{
				escenario.getObjetoPorNombre("hero").cambiarEstado(2);
			}
		}
		if (escenario.getObjetoPorNombre("hero").colisionInferior)
		{
			saltando = false;
		}
	}
}
