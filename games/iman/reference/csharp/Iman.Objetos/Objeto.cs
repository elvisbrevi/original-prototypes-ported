using Iman.Camaras;
using Iman.Controles;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Objetos;

internal abstract class Objeto
{
	public enum Diseño
	{
		NULL,
		_01,
		_02,
		_03,
		_04,
		_05,
		_06,
		_07,
		_08,
		_09,
		_10,
		_11,
		_12,
		_13,
		_14,
		_15,
		_16,
		_17,
		_18,
		_19
	}

	public enum Orientacion
	{
		Arriba,
		Abajo,
		Derecha,
		Izquierda
	}

	public enum Estado
	{
		Parado,
		Corriendo,
		Saltando,
		Cayendo
	}

	public enum Colision
	{
		Superior,
		Lateral,
		Inferior,
		Total,
		Ninguna
	}

	public enum Movimiento
	{
		Ninguno,
		Vertical,
		Horizontal
	}

	public enum TipoObjeto
	{
		Plataforma,
		Escalera,
		Personaje,
		Enemigo,
		Arma
	}

	public Diseño diseño;

	public Orientacion orientacion;

	public TipoObjeto tipoObjeto;

	public bool activo;

	public Colision colision;

	public Movimiento movimiento;

	public int posicionInicial;

	public int posicionFinal;

	public Estado estado;

	public Texture2D textura;

	public Color[] textureData;

	public Vector2 posicion;

	public Vector2 origen;

	public Rectangle cuerpo = default(Rectangle);

	public bool colisionSuperior;

	public bool colisionInferior;

	public bool colisionIzquierda;

	public bool colisionDerecha;

	public bool colisionEscalera;

	public int velocidad = 0;

	public int masa = 0;

	public bool ida;

	public bool vuelta;

	public float rotacion;

	public bool gravedad;

	public float fuerzaGravedad;

	public float rangoIman;

	public bool usandoIman;

	public Rectangle iman = default(Rectangle);

	public Vector2 puntoIman;

	public abstract void cargar(ContentManager content);

	public abstract void actualizar(GameTime tiempo, Control control);

	public abstract void dibujar(SpriteBatch sprite, Camera2D camara);
}
