using System.Collections.Generic;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Proyecto_Nutria;

internal abstract class Menu
{
	public bool activo;

	protected int cantidadControles;

	protected bool personajesActivos;

	public bool PersonajesActivos
	{
		get
		{
			return personajesActivos;
		}
		set
		{
			personajesActivos = value;
		}
	}

	public int CantidadControles
	{
		get
		{
			return cantidadControles;
		}
		set
		{
			cantidadControles = value;
		}
	}

	public abstract void cargar(ContentManager content, Nivel nivel);

	public abstract void actualizar(List<Control> controles, Nivel nivel, Camara camara);

	public abstract void dibujar(SpriteBatch sprite, Camara camara);
}
