using System.Collections.Generic;
using Iman.Camaras;
using Iman.Controles;
using Iman.Fisicas;
using Iman.Graficos;
using Iman.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Escenarios;

internal class Escenario_01 : Escenario
{
	public override void cargar(ContentManager content, Camera2D camara)
	{
		fisica = new Fisica();
		ID = 0;
		crearPlataforma(new Rectangle(0, 510, 600, 25), Objeto.Diseño.NULL, Objeto.Colision.Superior, content);
		crearPlataforma(new Rectangle(550, 400, 200, 150), Objeto.Diseño.NULL, Objeto.Colision.Total, content);
		crearPlataforma(new Rectangle(0, 300, 0, 0), Objeto.Diseño._02, Objeto.Colision.Superior, content);
		crearPlataforma(new Rectangle(100, 370, 0, 0), Objeto.Diseño._02, Objeto.Colision.Superior, content);
		crearPlataformaIman(new Vector2(550f, 300f), Objeto.Diseño._01, Objeto.Colision.Superior, content);
		crearPlataformaMovediza(new Vector2(40f, 0f), content, Objeto.Diseño._03, Objeto.Movimiento.Vertical, 220, 1);
		crearGrafico(Grafico.Diseño._01, Grafico.Tipo.Cielo, Vector2.Zero, 1f, content);
		crearGrafico(Grafico.Diseño._01, Grafico.Tipo.Background, new Vector2(0f, 250f), 1.1f, content);
		crearGrafico(Grafico.Diseño._01, Grafico.Tipo.Suelo, new Vector2(0f, 0f), 1.2f, content);
		crearGrafico(Grafico.Diseño._01, Grafico.Tipo.Front, new Vector2(0f, 550f), 1.5f, content);
		ramas = new List<Ramas>();
		ramas.Add(new Ramas());
		ramas[0].posicion = new Vector2(700f, 270f);
		ramas[0].rotacion = 0.5f;
		ramas[0].diseño = Ramas.Diseño._01;
		ramas[0].direccion = SpriteEffects.FlipVertically;
		ramas.Add(new Ramas());
		ramas[1].posicion = new Vector2(700f, 270f);
		ramas[1].rotacion = -0.1f;
		ramas[1].diseño = Ramas.Diseño._01;
		ramas[1].direccion = SpriteEffects.FlipHorizontally;
		foreach (Ramas rama in ramas)
		{
			rama.cargar(content);
		}
		crearNiño(new Vector2(50f, 390f), content);
		limites = new Rectangle(0, -100, 750, 480);
		camara.scale = 1f;
		camara.position = new Vector2(0f, 150f);
		camara.limites = new Rectangle(0, 0, 128, 95);
	}

	public override void actualizar(GameTime tiempo, Camera2D camara, Control control)
	{
		fisica.actualizar(objetos, tiempo);
		moverCamara(camara);
		foreach (Grafico grafico in graficos)
		{
			grafico.actualizar(camara, tiempo);
		}
		foreach (Ramas rama in ramas)
		{
			rama.actualizar(tiempo);
		}
		foreach (Objeto objeto in objetos)
		{
			objeto.actualizar(tiempo, control);
		}
		if (objetos[objetos.Count - 1].posicion.Y < 20f)
		{
			activo = false;
		}
	}

	public override void dibujar(SpriteBatch sprite, Camera2D camara, GraphicsDeviceManager graphics)
	{
		graficos[0].dibujar(sprite, camara);
		graficos[1].dibujar(sprite, camara);
		ramas[0].dibujar(sprite);
		graficos[2].dibujar(sprite, camara);
		ramas[1].dibujar(sprite);
		dibujarObjetos(sprite, camara);
		graficos[3].dibujar(sprite, camara);
	}

	private void dibujarObjetos(SpriteBatch sprite, Camera2D camara)
	{
		foreach (Objeto objeto in objetos)
		{
			if (objeto.diseño != Objeto.Diseño.NULL)
			{
				objeto.dibujar(sprite, camara);
			}
		}
	}
}
