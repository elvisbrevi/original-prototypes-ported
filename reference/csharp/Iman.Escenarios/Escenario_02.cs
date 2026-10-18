using Iman.Camaras;
using Iman.Controles;
using Iman.Fisicas;
using Iman.Graficos;
using Iman.Objetos;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Iman.Escenarios;

internal class Escenario_02 : Escenario
{
	public override void cargar(ContentManager content, Camera2D camara)
	{
		fisica = new Fisica();
		ID = 1;
		crearPlataforma(new Rectangle(0, 785, 600, 25), Objeto.Diseño.NULL, Objeto.Colision.Superior, content);
		crearPlataforma(new Rectangle(600, 600, 200, 400), Objeto.Diseño.NULL, Objeto.Colision.Total, content);
		crearPlataforma(new Rectangle(0, 520, 265, 25), Objeto.Diseño.NULL, Objeto.Colision.Superior, content);
		crearPlataforma(new Rectangle(190, 305, 270, 25), Objeto.Diseño.NULL, Objeto.Colision.Superior, content);
		crearPlataforma(new Rectangle(580, 225, 240, 25), Objeto.Diseño.NULL, Objeto.Colision.Superior, content);
		crearPlataformaIman(new Vector2(300f, 500f), Objeto.Diseño._04, Objeto.Colision.Superior, content);
		crearPlataformaMovediza(new Vector2(505f, 330f), content, Objeto.Diseño._03, Objeto.Movimiento.Vertical, 50, 1);
		crearPlataformaMovediza(new Vector2(300f, 320f), content, Objeto.Diseño._03, Objeto.Movimiento.Vertical, 70, 2);
		crearPlataformaMovediza(new Vector2(35f, 120f), content, Objeto.Diseño._03, Objeto.Movimiento.Vertical, 200, 2);
		crearGrafico(Grafico.Diseño._02, Grafico.Tipo.Cielo, Vector2.Zero, 1.1f, content);
		crearGrafico(Grafico.Diseño._02, Grafico.Tipo.Background, new Vector2(0f, 300f), 1.1f, content);
		crearGrafico(Grafico.Diseño._02, Grafico.Tipo.Suelo, new Vector2(0f, 120f), 1.2f, content);
		crearGrafico(Grafico.Diseño._02, Grafico.Tipo.Front, new Vector2(400f, 1000f), 0.9f, content);
		crearGrafico(Grafico.Diseño._02, Grafico.Tipo.Front, new Vector2(0f, 1040f), 0.9f, content);
		crearNiño(new Vector2(50f, 680f), content);
		limites = new Rectangle(0, -100, 750, 480);
		camara.scale = 1f;
		camara.position = new Vector2(0f, 150f);
		camara.limites = new Rectangle(0, -200, 128, 600);
	}

	public override void actualizar(GameTime tiempo, Camera2D camara, Control control)
	{
		fisica.actualizar(objetos, tiempo);
		moverCamara(camara);
		foreach (Grafico grafico in graficos)
		{
			grafico.actualizar(camara, tiempo);
		}
		foreach (Objeto objeto in objetos)
		{
			objeto.actualizar(tiempo, control);
		}
		if (objetos[objetos.Count - 1].posicion.Y < 150f && objetos[objetos.Count - 1].posicion.X > 660f)
		{
			activo = false;
		}
	}

	public override void dibujar(SpriteBatch sprite, Camera2D camara, GraphicsDeviceManager graphics)
	{
		graficos[0].dibujar(sprite, camara);
		graficos[1].dibujar(sprite, camara);
		graficos[2].dibujar(sprite, camara);
		dibujarObjetos(sprite, camara);
		graficos[3].dibujar(sprite, camara);
		graficos[4].dibujar(sprite, camara);
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
