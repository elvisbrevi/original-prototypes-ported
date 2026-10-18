using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal abstract class Personaje
{
	public enum Tipo
	{
		Coipo,
		Pollo
	}

	public enum Estado
	{
		Parado,
		Caminando,
		Saltando,
		Cayendo
	}

	public enum Orientacion
	{
		Derecha,
		Izquierda,
		Arriba,
		Abajo
	}

	public string nombre;

	public Tipo tipo;

	public string url = "Graficos/Personajes/";

	public Texture2D textura;

	public Vector2 Posicion;

	public Vector2 origen;

	public Rectangle Cuadro;

	public float rotacion;

	public float Zoom;

	protected SpriteEffects direccion;

	public Color color = Color.White;

	protected Orientacion orientacion = Orientacion.Derecha;

	public Rectangle Cuerpo;

	public bool ColisionSuperior;

	public bool ColisionInferior;

	public bool ColisionDerecha;

	public bool ColisionIzquierda;

	protected Color[] textureData;

	public float VelocidadMovimiento;

	public float velocidadRotacion;

	public float limiteSalto;

	public float velocidadSalto;

	public float cuadros;

	public float distanciaSalto;

	public float vida;

	protected bool girando;

	protected bool resbalando;

	public Rectangle limites;

	public int monedas;

	public Estado Estado_;

	protected float maxFrameRate;

	protected float frameRate;

	protected Vector2 framePerSeconds;

	public Generador polvo;

	public bool golpeado;

	public double duracionGolpe = 0.5;

	public int posicionGolpe;

	public Arma arma;

	public bool armado;

	public int cantidadBalas = 20;

	public abstract void cargar(ContentManager content);

	public abstract void actualizar(GameTime tiempo, Control control);

	public void dibujar(SpriteBatch sprite, Camara camara)
	{
		dibujarPersonaje(sprite, camara);
		polvo.dibujar(sprite);
	}

	protected void limitarMovimiento()
	{
		Posicion.X = MathHelper.Clamp(Posicion.X, limites.Left, limites.Right);
		Posicion.Y = MathHelper.Clamp(Posicion.Y, limites.Top, limites.Bottom);
		if (Posicion.X <= (float)limites.Left)
		{
			VelocidadMovimiento = 0f;
			Posicion.X = limites.Left + 1;
		}
		if (Posicion.X >= (float)limites.Right)
		{
			VelocidadMovimiento = 0f;
			Posicion.X = limites.Right - 1;
		}
	}

	protected void correr(Control control)
	{
		if (VelocidadMovimiento <= 0f && control.LeftHold)
		{
			direccion = SpriteEffects.FlipHorizontally;
		}
		else if (VelocidadMovimiento >= 0f && control.RightHold)
		{
			direccion = SpriteEffects.None;
		}
		if (control.RightHold && !ColisionDerecha && VelocidadMovimiento < 6f && !resbalando)
		{
			VelocidadMovimiento += 0.1f;
		}
		if (control.LeftHold && !ColisionIzquierda && VelocidadMovimiento > -6f && !resbalando)
		{
			VelocidadMovimiento -= 0.1f;
		}
		if (resbalando && !ColisionDerecha && !ColisionIzquierda)
		{
			if (VelocidadMovimiento > 0.1f)
			{
				VelocidadMovimiento -= 0.2f;
			}
			else if (VelocidadMovimiento < -0.1f)
			{
				VelocidadMovimiento += 0.2f;
			}
			else
			{
				VelocidadMovimiento = 0f;
			}
		}
		if (VelocidadMovimiento > 1f || (VelocidadMovimiento < -1f && !ColisionIzquierda && !ColisionDerecha))
		{
			Posicion.X += VelocidadMovimiento;
		}
	}

	protected void animar(GameTime gameTime, Control control)
	{
		if (ColisionSuperior && (control.LeftHold || control.RightHold))
		{
			polvo.Estado_ = Generador.Estado.Activo;
		}
		else
		{
			polvo.Estado_ = Generador.Estado.Inactivo;
		}
		polvo.actualizar(new Vector2(Posicion.X, Cuerpo.Bottom), gameTime);
		if (resbalando)
		{
			Cuadro.X = 0;
			Cuadro.Y = 70;
		}
		if (Estado_ == Estado.Saltando || (Estado_ == Estado.Cayendo && !ColisionSuperior))
		{
			if (girando)
			{
				rotacion += velocidadRotacion;
				Cuadro.X = 140;
				Cuadro.Y = 70;
				if (control.RightHold)
				{
					velocidadRotacion = 0.5f;
				}
				else if (control.LeftHold)
				{
					velocidadRotacion = -0.5f;
				}
			}
			else
			{
				Cuadro.X = 70;
				Cuadro.X = 70;
			}
		}
		if (ColisionSuperior)
		{
			rotacion = 0f;
			girando = false;
		}
		if (ColisionSuperior && !control.LeftHold && !control.RightHold)
		{
			Cuadro.X = 210;
			Cuadro.Y = 70;
		}
		if (!ColisionSuperior || (!control.LeftHold && !control.RightHold) || resbalando)
		{
			return;
		}
		Cuadro.Y = 0;
		if (frameRate > 70f)
		{
			frameRate = 0f;
			if ((float)Cuadro.X >= maxFrameRate - 70f)
			{
				Cuadro.X = -70;
			}
			Cuadro.X += 70;
		}
		frameRate += framePerSeconds.X * ((float)gameTime.ElapsedGameTime.TotalSeconds * 10f);
	}

	protected void dibujarPersonaje(SpriteBatch sprite, Camara camara)
	{
		sprite.Draw(textura, new Vector2(Posicion.X + origen.X, Posicion.Y + origen.Y), Cuadro, color, rotacion, origen, Zoom, direccion, 1f);
		if (armado)
		{
			arma.dibujar(sprite, camara);
		}
	}

	protected void cambiarEstados(Control control)
	{
		if ((VelocidadMovimiento > 0f && !control.RightHold) || (VelocidadMovimiento < 0f && !control.LeftHold))
		{
			resbalando = true;
		}
		else
		{
			resbalando = false;
		}
		if ((control.LeftHold || control.RightHold) && Estado_ != Estado.Cayendo)
		{
			girando = true;
		}
		if (Estado_ != Estado.Saltando)
		{
			if (control.LeftHold || control.RightHold)
			{
				Estado_ = Estado.Caminando;
			}
			if (!control.RightHold && !control.LeftHold && !resbalando)
			{
				Estado_ = Estado.Parado;
			}
		}
		if (!ColisionSuperior && Estado_ != Estado.Saltando)
		{
			Estado_ = Estado.Cayendo;
		}
	}

	protected void saltar(Control control)
	{
		if (control.B_)
		{
			limiteSalto = Posicion.Y - 250f;
			distanciaSalto = Posicion.Y;
			velocidadSalto = 8f;
		}
		if (control.BHold && Estado_ != Estado.Cayendo && !ColisionInferior)
		{
			if (distanciaSalto > limiteSalto)
			{
				distanciaSalto -= 8f;
				Estado_ = Estado.Saltando;
			}
			if (Estado_ == Estado.Saltando && Posicion.Y > distanciaSalto)
			{
				Posicion.Y -= velocidadSalto;
				if (velocidadSalto >= 0.3f)
				{
					velocidadSalto -= 0.3f;
				}
				if (velocidadSalto <= 0.5f)
				{
					Estado_ = Estado.Parado;
				}
			}
		}
		else
		{
			distanciaSalto = 0f;
			velocidadSalto = 0f;
			Estado_ = Estado.Cayendo;
		}
	}

	protected void disparar(Control control, GameTime tiempo)
	{
		if (!armado)
		{
			return;
		}
		arma.direccion = direccion;
		foreach (Bala bala in arma.balas)
		{
			if (!bala.activa)
			{
				if (direccion == SpriteEffects.None)
				{
					if (arma.nombre.Equals("Laser"))
					{
						bala.direccion = new Vector2(20f, 0f);
						bala.posicion = new Vector2(arma.posicion.X + 80f, arma.posicion.Y + 30f);
						arma.posicion = new Vector2(Posicion.X, Posicion.Y - 10f);
					}
					else if (arma.nombre.Equals("Lanzallamas"))
					{
						bala.direccion = new Vector2(20f, 0f);
						bala.posicion = new Vector2(arma.posicion.X + 80f, arma.posicion.Y + 30f);
						arma.posicion = new Vector2(Posicion.X, Posicion.Y - 10f);
					}
					bala.rotacion = 0f;
				}
				else if (direccion == SpriteEffects.FlipHorizontally)
				{
					if (arma.nombre.Equals("Laser"))
					{
						bala.direccion = new Vector2(-20f, 0f);
						bala.posicion = new Vector2(arma.posicion.X - (float)bala.textura.Width + 10f, arma.posicion.Y + 30f);
						arma.posicion = new Vector2(Posicion.X - 50f, Posicion.Y - 10f);
					}
					else if (arma.nombre.Equals("Lanzallamas"))
					{
						bala.direccion = new Vector2(-20f, 0f);
						bala.posicion = new Vector2(arma.posicion.X - (float)bala.textura.Width + 10f, arma.posicion.Y + 30f);
						arma.posicion = new Vector2(Posicion.X - 50f, Posicion.Y - 10f);
					}
					bala.rotacion = 0f;
				}
			}
			else
			{
				arma.disparar(bala, tiempo);
			}
		}
		if (control.A_ && arma.nombre.Equals("Laser") && cantidadBalas > 0 && arma.balas.Count > 0)
		{
			arma.balas[cantidadBalas - 1].activa = true;
			cantidadBalas--;
		}
		if (control.A_ && arma.nombre.Equals("Lanzallamas") && cantidadBalas > 0 && arma.balas.Count > 0)
		{
			arma.balas[cantidadBalas - 1].activa = true;
			cantidadBalas--;
		}
	}
}
