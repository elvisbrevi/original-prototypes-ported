using System;
using System.Collections.Generic;
using Engine.Datos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Objetos;

public class SistemaParticulas : Objeto
{
	private Random random;

	private List<Particula> particulas;

	public override void cargar(ContentManager Content)
	{
		Textura = Content.Load<Texture2D>("Graficos/" + NombreTextura);
		particulas = new List<Particula>();
		random = new Random();
	}

	public override void actualizar(GameTime tiempo)
	{
		if (Movimento)
		{
			mover();
		}
		int cantidadParticulas = CantidadParticulas;
		for (int i = 0; i < cantidadParticulas; i++)
		{
			if (base.Activo)
			{
				particulas.Add(generarNuevaParticula());
			}
		}
		for (int j = 0; j < particulas.Count; j++)
		{
			particulas[j].actualizar();
			if (particulas[j].TiempoVida <= 0)
			{
				particulas.RemoveAt(j);
				j--;
			}
		}
	}

	private Particula generarNuevaParticula()
	{
		Texture2D textura = Textura;
		Vector2 posicion = Posicion;
		if (base.DireccionRandom)
		{
			Direccion = new Vector2(1f * (float)(random.NextDouble() * 2.0 - 1.0), 1f * (float)(random.NextDouble() * 2.0 - 1.0));
		}
		float angulo = 0f;
		float velocidadAngular = 0.1f * (float)(random.NextDouble() * 2.0 - 1.0);
		if (base.ColorRandom)
		{
			Color_ = new Color((float)random.NextDouble(), (float)random.NextDouble(), (float)random.NextDouble());
		}
		int tiempoVida = 20 + random.Next(40);
		return new Particula(textura, posicion, Direccion, angulo, velocidadAngular, Color_, Tamaño, tiempoVida);
	}

	public override void dibujar(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		if (fueraDeCamara)
		{
			spriteBatch.Begin(blendMode, SpriteSortMode.Immediate, SaveStateMode.None);
		}
		else
		{
			spriteBatch.Begin(blendMode, SpriteSortMode.Immediate, SaveStateMode.None, Pantalla.getTransformationMatrix());
		}
		if (Escenario.ActivarShader && Escenario.Shader_ != Shader.Ninguno && !noShaderCamara)
		{
			Escenario.PostProcessing.iniciar(spriteBatch, graphics);
		}
		else if (Shader_ != Shader.Ninguno && !Escenario.ActivarShader)
		{
			PostProcessing.iniciar(spriteBatch, graphics);
		}
		for (int i = 0; i < particulas.Count; i++)
		{
			particulas[i].dibujar(spriteBatch);
		}
		if (Escenario.ActivarShader && Escenario.Shader_ != Shader.Ninguno && !noShaderCamara)
		{
			Escenario.PostProcessing.finalizar(spriteBatch, graphics);
		}
		else if (Shader_ != Shader.Ninguno && !Escenario.ActivarShader)
		{
			PostProcessing.finalizar(spriteBatch, graphics);
		}
		spriteBatch.End();
	}
}
