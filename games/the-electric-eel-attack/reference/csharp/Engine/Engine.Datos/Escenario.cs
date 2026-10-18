using System;
using System.Collections.Generic;
using System.Xml;
using Engine.Audios;
using Engine.Fisicas;
using Engine.Objetos;
using Engine.Shaders;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Content;
using Microsoft.Xna.Framework.Graphics;

namespace Engine.Datos;

public class Escenario
{
	public static Texture2D pixel;

	private SpriteFont fuente;

	public string nombre;

	public List<Objeto> objetos;

	public Color colorGrilla = Color.White;

	public static bool Pause;

	public static bool MostrarCuerpos;

	public static bool MostrarGrilla;

	public static bool MostrarDatos;

	public static bool ActivarShader;

	public static Post_Processing PostProcessing = null;

	public static Objeto.Shader Shader_ = Objeto.Shader.Ninguno;

	public static Color Color_ = Color.White;

	public static int Ancho;

	public static int Alto;

	public List<Sonido> sonidos = new List<Sonido>();

	private int frameRate = 0;

	private int frameCounter = 0;

	private TimeSpan elapsedTime = TimeSpan.Zero;

	private XmlDocument doc;

	public void guardarEscenario(string nombreEscenario)
	{
		//IL_0002: Unknown result type (might be due to invalid IL or missing references)
		//IL_000c: Expected O, but got Unknown
		doc = new XmlDocument();
		XmlNode val = (XmlNode)(object)doc.CreateElement("Escenario");
		string text = nombreEscenario;
		int count = 0;
		for (int i = 0; i < text.Length; i++)
		{
			if (text[i] == '\\')
			{
				count = i + 1;
			}
		}
		text = text.Remove(0, count);
		XmlAttribute val2 = doc.CreateAttribute("Nombre");
		((XmlNode)val2).InnerText = text;
		val.Attributes.Append(val2);
		XmlAttribute val3 = doc.CreateAttribute("ColorR");
		((XmlNode)val3).InnerText = Color_.R.ToString();
		val.Attributes.Append(val3);
		XmlAttribute val4 = doc.CreateAttribute("ColorG");
		((XmlNode)val4).InnerText = Color_.G.ToString();
		val.Attributes.Append(val4);
		XmlAttribute val5 = doc.CreateAttribute("ColorB");
		((XmlNode)val5).InnerText = Color_.B.ToString();
		val.Attributes.Append(val5);
		XmlAttribute val6 = doc.CreateAttribute("ColorA");
		((XmlNode)val6).InnerText = Color_.A.ToString();
		val.Attributes.Append(val6);
		XmlAttribute val7 = doc.CreateAttribute("MostrarGrilla");
		((XmlNode)val7).InnerText = MostrarGrilla.ToString();
		val.Attributes.Append(val7);
		XmlAttribute val8 = doc.CreateAttribute("MostrarDatos");
		((XmlNode)val8).InnerText = MostrarDatos.ToString();
		val.Attributes.Append(val8);
		XmlAttribute val9 = doc.CreateAttribute("MostrarCuerpos");
		((XmlNode)val9).InnerText = MostrarCuerpos.ToString();
		val.Attributes.Append(val9);
		XmlAttribute val10 = doc.CreateAttribute("Shaders");
		((XmlNode)val10).InnerText = Shader_.ToString();
		val.Attributes.Append(val10);
		XmlAttribute val11 = doc.CreateAttribute("X");
		((XmlNode)val11).InnerText = Pantalla.Posicion.X.ToString();
		val.Attributes.Append(val11);
		XmlAttribute val12 = doc.CreateAttribute("Y");
		((XmlNode)val12).InnerText = Pantalla.Posicion.Y.ToString();
		val.Attributes.Append(val12);
		XmlAttribute val13 = doc.CreateAttribute("Tamaño");
		((XmlNode)val13).InnerText = Pantalla.Tamaño.ToString();
		val.Attributes.Append(val13);
		XmlAttribute val14 = doc.CreateAttribute("Rotacion");
		((XmlNode)val14).InnerText = Pantalla.Inclinacion.ToString();
		val.Attributes.Append(val14);
		XmlAttribute val15 = doc.CreateAttribute("Ancho");
		((XmlNode)val15).InnerText = Pantalla._VWidth.ToString();
		val.Attributes.Append(val15);
		XmlAttribute val16 = doc.CreateAttribute("Alto");
		((XmlNode)val16).InnerText = Pantalla._VHeight.ToString();
		val.Attributes.Append(val16);
		foreach (Objeto objeto in objetos)
		{
			XmlNode val17 = (XmlNode)(object)doc.CreateElement(objeto.Nombre);
			XmlAttribute val18 = doc.CreateAttribute("Tipo");
			((XmlNode)val18).InnerText = objeto.Tipo_.ToString();
			val17.Attributes.Append(val18);
			XmlAttribute val19 = doc.CreateAttribute("Nombre");
			((XmlNode)val19).InnerText = objeto.Nombre;
			val17.Attributes.Append(val19);
			XmlAttribute val20 = doc.CreateAttribute("Nombre_Textura");
			((XmlNode)val20).InnerText = objeto.NombreTextura;
			val17.Attributes.Append(val20);
			XmlAttribute val21 = doc.CreateAttribute("PosicionX");
			((XmlNode)val21).InnerText = objeto.Posicion.X.ToString();
			val17.Attributes.Append(val21);
			XmlAttribute val22 = doc.CreateAttribute("PosicionY");
			((XmlNode)val22).InnerText = objeto.Posicion.Y.ToString();
			val17.Attributes.Append(val22);
			XmlAttribute val23 = doc.CreateAttribute("Velocidad");
			((XmlNode)val23).InnerText = objeto.Velocidad.ToString();
			val17.Attributes.Append(val23);
			XmlAttribute val24 = doc.CreateAttribute("DireccionY");
			((XmlNode)val24).InnerText = objeto.Direccion.Y.ToString();
			val17.Attributes.Append(val24);
			XmlAttribute val25 = doc.CreateAttribute("DireccionX");
			((XmlNode)val25).InnerText = objeto.Direccion.X.ToString();
			val17.Attributes.Append(val25);
			XmlAttribute val26 = doc.CreateAttribute("Colision");
			((XmlNode)val26).InnerText = objeto.Colision_.ToString();
			val17.Attributes.Append(val26);
			XmlAttribute val27 = doc.CreateAttribute("Inclinacion");
			((XmlNode)val27).InnerText = objeto.Inclinacion.ToString();
			val17.Attributes.Append(val27);
			XmlAttribute val28 = doc.CreateAttribute("CantidadParticulas");
			((XmlNode)val28).InnerText = objeto.CantidadParticulas.ToString();
			val17.Attributes.Append(val28);
			XmlAttribute val29 = doc.CreateAttribute("RepetirScroll");
			((XmlNode)val29).InnerText = objeto.repetirScroll.ToString();
			val17.Attributes.Append(val29);
			XmlAttribute val30 = doc.CreateAttribute("Activo");
			((XmlNode)val30).InnerText = objeto.Activo.ToString();
			val17.Attributes.Append(val30);
			XmlAttribute val31 = doc.CreateAttribute("NoShaderCamara");
			((XmlNode)val31).InnerText = objeto.noShaderCamara.ToString();
			val17.Attributes.Append(val31);
			XmlAttribute val32 = doc.CreateAttribute("ColorRandom");
			((XmlNode)val32).InnerText = objeto.ColorRandom.ToString();
			val17.Attributes.Append(val32);
			XmlAttribute val33 = doc.CreateAttribute("TamañoRandom");
			((XmlNode)val33).InnerText = objeto.TamañoRandom.ToString();
			val17.Attributes.Append(val33);
			XmlAttribute val34 = doc.CreateAttribute("DireccionRandom");
			((XmlNode)val34).InnerText = objeto.DireccionRandom.ToString();
			val17.Attributes.Append(val34);
			XmlAttribute val35 = doc.CreateAttribute("Texto");
			try
			{
				((XmlNode)val35).InnerText = objeto.Texto.ToString();
			}
			catch (NullReferenceException)
			{
				((XmlNode)val35).InnerText = "";
			}
			val17.Attributes.Append(val35);
			XmlAttribute val36 = doc.CreateAttribute("Gravedad");
			((XmlNode)val36).InnerText = objeto.Gravedad.ToString();
			val17.Attributes.Append(val36);
			XmlAttribute val37 = doc.CreateAttribute("FueraDeCamara");
			((XmlNode)val37).InnerText = objeto.fueraDeCamara.ToString();
			val17.Attributes.Append(val37);
			XmlAttribute val38 = doc.CreateAttribute("Movimento");
			((XmlNode)val38).InnerText = objeto.Movimento.ToString();
			val17.Attributes.Append(val38);
			XmlAttribute val39 = doc.CreateAttribute("BlendMode");
			((XmlNode)val39).InnerText = objeto.blendMode.ToString();
			val17.Attributes.Append(val39);
			XmlNode val40 = (XmlNode)(object)doc.CreateElement("Sonidos");
			foreach (Sonido sonido in objeto.Sonidos)
			{
				XmlNode val41 = (XmlNode)(object)doc.CreateElement(sonido.Nombre);
				XmlAttribute val42 = doc.CreateAttribute("Nombre");
				((XmlNode)val42).InnerText = sonido.Nombre.ToString();
				val41.Attributes.Append(val42);
				XmlAttribute val43 = doc.CreateAttribute("Volumen");
				((XmlNode)val43).InnerText = sonido.instanciaSonido.Volume.ToString();
				val41.Attributes.Append(val43);
				val40.AppendChild(val41);
			}
			val17.AppendChild(val40);
			XmlAttribute val44 = doc.CreateAttribute("Animado");
			((XmlNode)val44).InnerText = objeto.Animado.ToString();
			val17.Attributes.Append(val44);
			if (objeto.Animado)
			{
				XmlNode val45 = (XmlNode)(object)doc.CreateElement("Estados");
				XmlAttribute val46 = doc.CreateAttribute("EstadoSeleccionado");
				((XmlNode)val46).InnerText = objeto.EstadoSeleccionado.ToString();
				val45.Attributes.Append(val46);
				foreach (Estado estado in objeto.Estados)
				{
					XmlNode val47 = (XmlNode)(object)doc.CreateElement(estado.Nombre);
					XmlAttribute val42 = doc.CreateAttribute("Nombre");
					((XmlNode)val42).InnerText = estado.Nombre.ToString();
					val47.Attributes.Append(val42);
					XmlAttribute val48 = doc.CreateAttribute("CuadroX");
					((XmlNode)val48).InnerText = estado.Cuadro.X.ToString();
					val47.Attributes.Append(val48);
					XmlAttribute val49 = doc.CreateAttribute("CuadroY");
					((XmlNode)val49).InnerText = estado.Cuadro.Y.ToString();
					val47.Attributes.Append(val49);
					XmlAttribute val50 = doc.CreateAttribute("CuadroAncho");
					((XmlNode)val50).InnerText = estado.Cuadro.Width.ToString();
					val47.Attributes.Append(val50);
					XmlAttribute val51 = doc.CreateAttribute("CuadroAlto");
					((XmlNode)val51).InnerText = estado.Cuadro.Height.ToString();
					val47.Attributes.Append(val51);
					XmlAttribute val52 = doc.CreateAttribute("FPS");
					((XmlNode)val52).InnerText = estado.FPS.ToString();
					val47.Attributes.Append(val52);
					XmlAttribute val53 = doc.CreateAttribute("Numero");
					((XmlNode)val53).InnerText = estado.Numero.ToString();
					val47.Attributes.Append(val53);
					XmlAttribute val54 = doc.CreateAttribute("NoRepetir");
					((XmlNode)val54).InnerText = estado.NoRepetir.ToString();
					val47.Attributes.Append(val54);
					XmlAttribute val55 = doc.CreateAttribute("MinVariacion");
					((XmlNode)val55).InnerText = estado.MinVariacion.ToString();
					val47.Attributes.Append(val55);
					XmlAttribute val56 = doc.CreateAttribute("MaxVariacion");
					((XmlNode)val56).InnerText = estado.MaxVariacion.ToString();
					val47.Attributes.Append(val56);
					XmlAttribute val57 = doc.CreateAttribute("VelocidadVariacion");
					((XmlNode)val57).InnerText = estado.VelocidadVariacion.ToString();
					val47.Attributes.Append(val57);
					XmlAttribute val58 = doc.CreateAttribute("Variacion");
					((XmlNode)val58).InnerText = estado.Variacion_.ToString();
					val47.Attributes.Append(val58);
					val45.AppendChild(val47);
				}
				val17.AppendChild(val45);
			}
			XmlAttribute val59 = doc.CreateAttribute("ScrollHorizontal");
			((XmlNode)val59).InnerText = objeto.ScrollHorizontal.ToString();
			val17.Attributes.Append(val59);
			XmlAttribute val60 = doc.CreateAttribute("ScrollVertical");
			((XmlNode)val60).InnerText = objeto.ScrollVertical.ToString();
			val17.Attributes.Append(val60);
			XmlAttribute val61 = doc.CreateAttribute("Tamaño");
			((XmlNode)val61).InnerText = objeto.Tamaño.ToString();
			val17.Attributes.Append(val61);
			XmlAttribute val62 = doc.CreateAttribute("InicioX");
			((XmlNode)val62).InnerText = objeto.Inicio.X.ToString();
			val17.Attributes.Append(val62);
			XmlAttribute val63 = doc.CreateAttribute("InicioY");
			((XmlNode)val63).InnerText = objeto.Inicio.Y.ToString();
			val17.Attributes.Append(val63);
			XmlAttribute val64 = doc.CreateAttribute("DestinoX");
			((XmlNode)val64).InnerText = objeto.Destino.X.ToString();
			val17.Attributes.Append(val64);
			XmlAttribute val65 = doc.CreateAttribute("DestinoY");
			((XmlNode)val65).InnerText = objeto.Destino.Y.ToString();
			val17.Attributes.Append(val65);
			XmlAttribute val66 = doc.CreateAttribute("ColorR");
			((XmlNode)val66).InnerText = objeto.Color_.R.ToString();
			val17.Attributes.Append(val66);
			XmlAttribute val67 = doc.CreateAttribute("ColorG");
			((XmlNode)val67).InnerText = objeto.Color_.G.ToString();
			val17.Attributes.Append(val67);
			XmlAttribute val68 = doc.CreateAttribute("ColorB");
			((XmlNode)val68).InnerText = objeto.Color_.B.ToString();
			val17.Attributes.Append(val68);
			XmlAttribute val69 = doc.CreateAttribute("ColorA");
			((XmlNode)val69).InnerText = objeto.Color_.A.ToString();
			val17.Attributes.Append(val69);
			XmlAttribute val70 = doc.CreateAttribute("SpriteEffects");
			((XmlNode)val70).InnerText = objeto.SpriteEffect_.ToString();
			val17.Attributes.Append(val70);
			XmlAttribute val71 = doc.CreateAttribute("Lugar");
			((XmlNode)val71).InnerText = objeto.Lugar_.ToString();
			val17.Attributes.Append(val71);
			switch (objeto.Tipo_)
			{
			case Objeto.Tipo.Grafico:
			{
				XmlAttribute val72 = doc.CreateAttribute("CuerpoX");
				((XmlNode)val72).InnerText = objeto.Posicion.X.ToString();
				val17.Attributes.Append(val72);
				XmlAttribute val73 = doc.CreateAttribute("CuerpoY");
				((XmlNode)val73).InnerText = objeto.Posicion.Y.ToString();
				val17.Attributes.Append(val73);
				if (!objeto.Animado)
				{
					XmlAttribute val74 = doc.CreateAttribute("CuerpoAncho");
					((XmlNode)val74).InnerText = ((int)((float)objeto.Textura.Width * objeto.Tamaño)).ToString();
					val17.Attributes.Append(val74);
					XmlAttribute val75 = doc.CreateAttribute("CuerpoAlto");
					((XmlNode)val75).InnerText = ((int)((float)objeto.Textura.Height * objeto.Tamaño)).ToString();
					val17.Attributes.Append(val75);
				}
				else
				{
					XmlAttribute val74 = doc.CreateAttribute("CuerpoAncho");
					((XmlNode)val74).InnerText = ((float)objeto.Estados[objeto.EstadoSeleccionado].Cuadro.Width * objeto.Tamaño).ToString();
					val17.Attributes.Append(val74);
					XmlAttribute val75 = doc.CreateAttribute("CuerpoAlto");
					((XmlNode)val75).InnerText = ((float)objeto.Estados[objeto.EstadoSeleccionado].Cuadro.Height * objeto.Tamaño).ToString();
					val17.Attributes.Append(val75);
				}
				break;
			}
			case Objeto.Tipo.Plataforma:
			{
				XmlAttribute val72 = doc.CreateAttribute("CuerpoX");
				((XmlNode)val72).InnerText = objeto.Posicion.X.ToString();
				val17.Attributes.Append(val72);
				XmlAttribute val73 = doc.CreateAttribute("CuerpoY");
				((XmlNode)val73).InnerText = objeto.Posicion.Y.ToString();
				val17.Attributes.Append(val73);
				XmlAttribute val74 = doc.CreateAttribute("CuerpoAncho");
				((XmlNode)val74).InnerText = objeto.Cuerpo.Width.ToString();
				val17.Attributes.Append(val74);
				XmlAttribute val75 = doc.CreateAttribute("CuerpoAlto");
				((XmlNode)val75).InnerText = objeto.Cuerpo.Height.ToString();
				val17.Attributes.Append(val75);
				break;
			}
			case Objeto.Tipo.Personaje:
			{
				XmlAttribute val72 = doc.CreateAttribute("CuerpoX");
				((XmlNode)val72).InnerText = objeto.Posicion.X.ToString();
				val17.Attributes.Append(val72);
				XmlAttribute val73 = doc.CreateAttribute("CuerpoY");
				((XmlNode)val73).InnerText = objeto.Posicion.Y.ToString();
				val17.Attributes.Append(val73);
				if (!objeto.Animado)
				{
					XmlAttribute val74 = doc.CreateAttribute("CuerpoAncho");
					((XmlNode)val74).InnerText = ((int)((float)objeto.Textura.Width * objeto.Tamaño)).ToString();
					val17.Attributes.Append(val74);
					XmlAttribute val75 = doc.CreateAttribute("CuerpoAlto");
					((XmlNode)val75).InnerText = ((int)((float)objeto.Textura.Height * objeto.Tamaño)).ToString();
					val17.Attributes.Append(val75);
				}
				else
				{
					XmlAttribute val74 = doc.CreateAttribute("CuerpoAncho");
					((XmlNode)val74).InnerText = ((float)objeto.Estados[objeto.EstadoSeleccionado].Cuadro.Width * objeto.Tamaño).ToString();
					val17.Attributes.Append(val74);
					XmlAttribute val75 = doc.CreateAttribute("CuerpoAlto");
					((XmlNode)val75).InnerText = ((float)objeto.Estados[objeto.EstadoSeleccionado].Cuadro.Height * objeto.Tamaño).ToString();
					val17.Attributes.Append(val75);
				}
				break;
			}
			}
			XmlNode val76 = (XmlNode)(object)doc.CreateElement("Shader");
			XmlAttribute val77 = doc.CreateAttribute("Tipo_Shader");
			((XmlNode)val77).InnerText = objeto.Shader_.ToString();
			val76.Attributes.Append(val77);
			if (objeto.Shader_ == Objeto.Shader.Filter)
			{
				XmlAttribute val78 = doc.CreateAttribute("burn");
				((XmlNode)val78).InnerText = objeto.PostProcessing.efecto.Parameters["burn"].GetValueSingle().ToString();
				val76.Attributes.Append(val78);
				XmlAttribute val79 = doc.CreateAttribute("saturation");
				((XmlNode)val79).InnerText = objeto.PostProcessing.efecto.Parameters["saturation"].GetValueSingle().ToString();
				val76.Attributes.Append(val79);
				XmlAttribute val80 = doc.CreateAttribute("r");
				((XmlNode)val80).InnerText = objeto.PostProcessing.efecto.Parameters["r"].GetValueSingle().ToString();
				val76.Attributes.Append(val80);
				XmlAttribute val81 = doc.CreateAttribute("g");
				((XmlNode)val81).InnerText = objeto.PostProcessing.efecto.Parameters["g"].GetValueSingle().ToString();
				val76.Attributes.Append(val81);
				XmlAttribute val82 = doc.CreateAttribute("b");
				((XmlNode)val82).InnerText = objeto.PostProcessing.efecto.Parameters["b"].GetValueSingle().ToString();
				val76.Attributes.Append(val82);
				XmlAttribute val83 = doc.CreateAttribute("brite");
				((XmlNode)val83).InnerText = objeto.PostProcessing.efecto.Parameters["brite"].GetValueSingle().ToString();
				val76.Attributes.Append(val83);
			}
			else if (objeto.Shader_ == Objeto.Shader.Brightness || objeto.Shader_ == Objeto.Shader.Fishbowl || objeto.Shader_ == Objeto.Shader.Grayscale)
			{
				XmlAttribute val84 = doc.CreateAttribute("Intensity");
				((XmlNode)val84).InnerText = objeto.PostProcessing.efecto.Parameters["Intensity"].GetValueSingle().ToString();
				val76.Attributes.Append(val84);
			}
			val17.AppendChild(val76);
			val.AppendChild(val17);
		}
		((XmlNode)doc).AppendChild(val);
		doc.Save(nombreEscenario);
	}

	public List<Objeto> retornarObjetos(string rutaXml)
	{
		//IL_0007: Unknown result type (might be due to invalid IL or missing references)
		//IL_000d: Expected O, but got Unknown
		//IL_0444: Unknown result type (might be due to invalid IL or missing references)
		//IL_044a: Expected O, but got Unknown
		//IL_046b: Unknown result type (might be due to invalid IL or missing references)
		//IL_0472: Expected O, but got Unknown
		//IL_0ddc: Unknown result type (might be due to invalid IL or missing references)
		//IL_0de3: Expected O, but got Unknown
		//IL_14e8: Unknown result type (might be due to invalid IL or missing references)
		//IL_14ef: Expected O, but got Unknown
		//IL_11ce: Unknown result type (might be due to invalid IL or missing references)
		//IL_11d5: Expected O, but got Unknown
		//IL_1531: Unknown result type (might be due to invalid IL or missing references)
		//IL_1538: Expected O, but got Unknown
		//IL_1237: Unknown result type (might be due to invalid IL or missing references)
		//IL_123e: Expected O, but got Unknown
		List<Objeto> list = new List<Objeto>();
		XmlDocument val = new XmlDocument();
		val.Load(rutaXml);
		nombre = ((XmlNode)((XmlNode)val).FirstChild.Attributes[0]).InnerText;
		Color_.R = byte.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[1]).InnerText);
		Color_.G = byte.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[2]).InnerText);
		Color_.B = byte.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[3]).InnerText);
		Color_.A = byte.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[4]).InnerText);
		MostrarGrilla = bool.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[5]).InnerText);
		MostrarDatos = bool.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[6]).InnerText);
		MostrarCuerpos = bool.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[7]).InnerText);
		switch (((XmlNode)((XmlNode)val).FirstChild.Attributes[8]).InnerText)
		{
		case "Ninguno":
			Shader_ = Objeto.Shader.Ninguno;
			PostProcessing = null;
			break;
		case "Blur":
			Shader_ = Objeto.Shader.Blur;
			PostProcessing = new Blur();
			break;
		case "Blur_Grayscale":
			Shader_ = Objeto.Shader.Blur_Grayscale;
			PostProcessing = new Blur_Grayscale();
			break;
		case "Brightness":
			Shader_ = Objeto.Shader.Brightness;
			PostProcessing = new Brightness();
			break;
		case "ColorSketch":
			Shader_ = Objeto.Shader.ColorSketch;
			PostProcessing = new ColorSketch();
			break;
		case "Filter":
			Shader_ = Objeto.Shader.Filter;
			PostProcessing = new Filter();
			break;
		case "Fishbowl":
			Shader_ = Objeto.Shader.Fishbowl;
			PostProcessing = new Fishbowl();
			break;
		case "Grayscale":
			Shader_ = Objeto.Shader.Grayscale;
			PostProcessing = new Grayscale();
			break;
		case "LeftFade":
			Shader_ = Objeto.Shader.LeftFade;
			PostProcessing = new LeftFade();
			break;
		case "MonoSketch":
			Shader_ = Objeto.Shader.MonoSketch;
			PostProcessing = new MonoSketch();
			break;
		case "Negative":
			Shader_ = Objeto.Shader.Negative;
			PostProcessing = new Negative();
			break;
		case "TopFade":
			Shader_ = Objeto.Shader.TopFade;
			PostProcessing = new TopFade();
			break;
		case "TopLeftFade":
			Shader_ = Objeto.Shader.TopLeftFade;
			PostProcessing = new TopLeftFade();
			break;
		}
		Pantalla.Posicion.X = int.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[9]).InnerText);
		Pantalla.Posicion.Y = int.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[10]).InnerText);
		Pantalla.Tamaño = float.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[11]).InnerText);
		Pantalla.Inclinacion = float.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[12]).InnerText);
		Ancho = int.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[13]).InnerText);
		Alto = int.Parse(((XmlNode)((XmlNode)val).FirstChild.Attributes[14]).InnerText);
		foreach (XmlNode childNode in ((XmlNode)val).FirstChild.ChildNodes)
		{
			XmlNode val2 = childNode;
			Objeto objeto = new Plataforma();
			foreach (XmlAttribute item in (XmlNamedNodeMap)val2.Attributes)
			{
				XmlAttribute val3 = item;
				switch (((XmlNode)val3).Name)
				{
				case "Tipo":
					if (((XmlNode)val3).InnerText == "Grafico")
					{
						objeto = new Grafico();
						objeto.Tipo_ = Objeto.Tipo.Grafico;
					}
					if (((XmlNode)val3).InnerText == "Plataforma")
					{
						objeto = new Plataforma();
						objeto.Tipo_ = Objeto.Tipo.Plataforma;
					}
					if (((XmlNode)val3).InnerText == "Personaje")
					{
						objeto = new Personaje();
						objeto.Tipo_ = Objeto.Tipo.Personaje;
					}
					if (((XmlNode)val3).InnerText == "Particulas")
					{
						objeto = new SistemaParticulas();
						objeto.Tipo_ = Objeto.Tipo.Particulas;
					}
					if (((XmlNode)val3).InnerText == "Texto")
					{
						objeto = new Texto();
						objeto.Tipo_ = Objeto.Tipo.Texto;
					}
					break;
				case "Nombre":
					objeto.Nombre = ((XmlNode)val3).InnerText;
					break;
				case "Texto":
					objeto.Texto = ((XmlNode)val3).InnerText;
					break;
				case "Nombre_Textura":
					objeto.NombreTextura = ((XmlNode)val3).InnerText;
					break;
				case "PosicionX":
					objeto.Posicion.X = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "PosicionY":
					objeto.Posicion.Y = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "DireccionX":
					objeto.Direccion.X = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "DireccionY":
					objeto.Direccion.Y = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "Colision":
					if (((XmlNode)val3).InnerText == "Inferior")
					{
						objeto.Colision_ = Objeto.Colision.Inferior;
					}
					if (((XmlNode)val3).InnerText == "Total")
					{
						objeto.Colision_ = Objeto.Colision.Total;
					}
					if (((XmlNode)val3).InnerText == "Superior")
					{
						objeto.Colision_ = Objeto.Colision.Superior;
					}
					if (((XmlNode)val3).InnerText == "Ninguna")
					{
						objeto.Colision_ = Objeto.Colision.Ninguna;
					}
					if (((XmlNode)val3).InnerText == "Lateral")
					{
						objeto.Colision_ = Objeto.Colision.Lateral;
					}
					if (((XmlNode)val3).InnerText == "Pixel")
					{
						objeto.Colision_ = Objeto.Colision.Pixel;
					}
					break;
				case "Velocidad":
					objeto.Velocidad = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "Inclinacion":
					objeto.Inclinacion = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "RepetirScroll":
					objeto.repetirScroll = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "Activo":
					objeto.Activo = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "NoShaderCamara":
					objeto.noShaderCamara = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "ColorRandom":
					objeto.ColorRandom = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "TamañoRandom":
					objeto.TamañoRandom = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "DireccionRandom":
					objeto.DireccionRandom = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "Gravedad":
					objeto.Gravedad = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "FueraDeCamara":
					objeto.fueraDeCamara = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "Movimento":
					objeto.Movimento = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "BlendMode":
					if (((XmlNode)val3).InnerText == "AlphaBlend")
					{
						objeto.blendMode = SpriteBlendMode.AlphaBlend;
					}
					else if (((XmlNode)val3).InnerText == "Additive")
					{
						objeto.blendMode = SpriteBlendMode.Additive;
					}
					break;
				case "Animado":
					objeto.Animado = bool.Parse(((XmlNode)val3).InnerText);
					break;
				case "ScrollHorizontal":
					objeto.ScrollHorizontal = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "ScrollVertical":
					objeto.ScrollVertical = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "CantidadParticulas":
					objeto.CantidadParticulas = int.Parse(((XmlNode)val3).InnerText);
					break;
				case "Tamaño":
					objeto.Tamaño = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "InicioX":
					objeto.Inicio.X = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "InicioY":
					objeto.Inicio.Y = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "DestinoX":
					objeto.Destino.X = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "DestinoY":
					objeto.Destino.Y = float.Parse(((XmlNode)val3).InnerText);
					break;
				case "ColorR":
					objeto.Color_.R = byte.Parse(((XmlNode)val3).InnerText);
					break;
				case "ColorG":
					objeto.Color_.G = byte.Parse(((XmlNode)val3).InnerText);
					break;
				case "ColorB":
					objeto.Color_.B = byte.Parse(((XmlNode)val3).InnerText);
					break;
				case "ColorA":
					objeto.Color_.A = byte.Parse(((XmlNode)val3).InnerText);
					break;
				case "SpriteEffects":
					if (((XmlNode)val3).InnerText == "FlipHorizontally")
					{
						objeto.SpriteEffect_ = SpriteEffects.FlipHorizontally;
					}
					if (((XmlNode)val3).InnerText == "FlipVertically")
					{
						objeto.SpriteEffect_ = SpriteEffects.FlipVertically;
					}
					if (((XmlNode)val3).InnerText == "None")
					{
						objeto.SpriteEffect_ = SpriteEffects.None;
					}
					break;
				case "Lugar":
					if (((XmlNode)val3).InnerText == "Fondo")
					{
						objeto.Lugar_ = Objeto.Lugar.Fondo;
					}
					if (((XmlNode)val3).InnerText == "Frente")
					{
						objeto.Lugar_ = Objeto.Lugar.Frente;
					}
					if (((XmlNode)val3).InnerText == "Medio")
					{
						objeto.Lugar_ = Objeto.Lugar.Medio;
					}
					break;
				case "CuerpoX":
					objeto.Cuerpo.X = (int)float.Parse(((XmlNode)val3).InnerText);
					break;
				case "CuerpoY":
					objeto.Cuerpo.Y = (int)float.Parse(((XmlNode)val3).InnerText);
					break;
				case "CuerpoAlto":
					objeto.Cuerpo.Height = (int)float.Parse(((XmlNode)val3).InnerText);
					break;
				case "CuerpoAncho":
					objeto.Cuerpo.Width = (int)float.Parse(((XmlNode)val3).InnerText);
					break;
				}
			}
			foreach (XmlNode item2 in val2)
			{
				XmlNode val4 = item2;
				if (val4.Name == "Shader")
				{
					switch (((XmlNode)val4.Attributes[0]).InnerText)
					{
					case "Ninguno":
						objeto.Shader_ = Objeto.Shader.Ninguno;
						objeto.PostProcessing = null;
						break;
					case "Filter":
						objeto.Shader_ = Objeto.Shader.Filter;
						objeto.PostProcessing = new Filter();
						break;
					case "Negative":
						objeto.Shader_ = Objeto.Shader.Negative;
						objeto.PostProcessing = new Negative();
						break;
					case "Blur_Grayscale":
						objeto.Shader_ = Objeto.Shader.Blur_Grayscale;
						objeto.PostProcessing = new Blur_Grayscale();
						break;
					case "ColorSketch":
						objeto.Shader_ = Objeto.Shader.ColorSketch;
						objeto.PostProcessing = new ColorSketch();
						break;
					case "MonoSketch":
						objeto.Shader_ = Objeto.Shader.MonoSketch;
						objeto.PostProcessing = new MonoSketch();
						break;
					case "Brightness":
						objeto.Shader_ = Objeto.Shader.Brightness;
						objeto.PostProcessing = new Brightness();
						break;
					case "TopFade":
						objeto.Shader_ = Objeto.Shader.TopFade;
						objeto.PostProcessing = new TopFade();
						break;
					case "LeftFade":
						objeto.Shader_ = Objeto.Shader.LeftFade;
						objeto.PostProcessing = new LeftFade();
						break;
					case "TopLeftFade":
						objeto.Shader_ = Objeto.Shader.TopLeftFade;
						objeto.PostProcessing = new TopLeftFade();
						break;
					case "Fishbowl":
						objeto.Shader_ = Objeto.Shader.Fishbowl;
						objeto.PostProcessing = new Fishbowl();
						break;
					case "Blur":
						objeto.Shader_ = Objeto.Shader.Blur;
						objeto.PostProcessing = new Blur();
						break;
					case "Grayscale":
						objeto.Shader_ = Objeto.Shader.Grayscale;
						objeto.PostProcessing = new Grayscale();
						break;
					}
					if (objeto.Shader_ == Objeto.Shader.Filter)
					{
						objeto.PostProcessing.burn = float.Parse(((XmlNode)val4.Attributes[1]).InnerText);
						objeto.PostProcessing.saturation = float.Parse(((XmlNode)val4.Attributes[2]).InnerText);
						objeto.PostProcessing.r = float.Parse(((XmlNode)val4.Attributes[3]).InnerText);
						objeto.PostProcessing.g = float.Parse(((XmlNode)val4.Attributes[4]).InnerText);
						objeto.PostProcessing.b = float.Parse(((XmlNode)val4.Attributes[5]).InnerText);
						objeto.PostProcessing.brite = float.Parse(((XmlNode)val4.Attributes[6]).InnerText);
					}
					else if (objeto.Shader_ == Objeto.Shader.Brightness || objeto.Shader_ == Objeto.Shader.Fishbowl || objeto.Shader_ == Objeto.Shader.Grayscale)
					{
						objeto.PostProcessing.intensidad = float.Parse(((XmlNode)val4.Attributes[1]).InnerText);
					}
				}
			}
			if (objeto.Animado)
			{
				foreach (XmlNode item3 in val2)
				{
					XmlNode val5 = item3;
					if (!(val5.Name == "Estados"))
					{
						continue;
					}
					objeto.EstadoSeleccionado = int.Parse(((XmlNode)val5.Attributes[0]).InnerText);
					objeto.Estados = new List<Estado>();
					foreach (XmlNode childNode2 in val5.ChildNodes)
					{
						XmlNode val6 = childNode2;
						Estado estado = new Estado();
						estado.Nombre = ((XmlNode)val6.Attributes[0]).InnerText;
						estado.Cuadro.X = int.Parse(((XmlNode)val6.Attributes[1]).InnerText);
						estado.Cuadro.Y = int.Parse(((XmlNode)val6.Attributes[2]).InnerText);
						estado.Cuadro.Width = int.Parse(((XmlNode)val6.Attributes[3]).InnerText);
						estado.Cuadro.Height = int.Parse(((XmlNode)val6.Attributes[4]).InnerText);
						estado.FPS = float.Parse(((XmlNode)val6.Attributes[5]).InnerText);
						estado.Numero = float.Parse(((XmlNode)val6.Attributes[6]).InnerText);
						estado.NoRepetir = bool.Parse(((XmlNode)val6.Attributes[7]).InnerText);
						estado.MinVariacion = float.Parse(((XmlNode)val6.Attributes[8]).InnerText);
						estado.MaxVariacion = float.Parse(((XmlNode)val6.Attributes[9]).InnerText);
						estado.VelocidadVariacion = float.Parse(((XmlNode)val6.Attributes[10]).InnerText);
						if (((XmlNode)val6.Attributes[11]).InnerText == "Ninguna")
						{
							estado.Variacion_ = Estado.Variacion.Ninguna;
						}
						if (((XmlNode)val6.Attributes[11]).InnerText == "Shader")
						{
							estado.Variacion_ = Estado.Variacion.Shader;
						}
						if (((XmlNode)val6.Attributes[11]).InnerText == "Rotacion")
						{
							estado.Variacion_ = Estado.Variacion.Rotacion;
						}
						if (((XmlNode)val6.Attributes[11]).InnerText == "Tamaño")
						{
							estado.Variacion_ = Estado.Variacion.Tamaño;
						}
						objeto.Estados.Add(estado);
					}
				}
			}
			foreach (XmlNode item4 in val2)
			{
				XmlNode val7 = item4;
				if (!(val7.Name == "Sonidos"))
				{
					continue;
				}
				objeto.Sonidos = new List<Sonido>();
				foreach (XmlNode childNode3 in val7.ChildNodes)
				{
					XmlNode val8 = childNode3;
					Sonido sonido = new Sonido();
					sonido.Nombre = ((XmlNode)val8.Attributes[0]).InnerText;
					sonido.Volumen = float.Parse(((XmlNode)val8.Attributes[1]).InnerText);
					objeto.Sonidos.Add(sonido);
				}
			}
			list.Add(objeto);
		}
		return list;
	}

	public void cargarObjetos(ContentManager content)
	{
		fuente = content.Load<SpriteFont>("Fuentes/normal");
		pixel = content.Load<Texture2D>("Graficos/pixel");
		foreach (Objeto objeto in objetos)
		{
			objeto.cargar(content);
			objeto.cargarSonidos(content);
			if (Shader_ != Objeto.Shader.Ninguno)
			{
				PostProcessing.cargar(content);
			}
			if (objeto.Shader_ != Objeto.Shader.Ninguno)
			{
				objeto.PostProcessing.cargar(content);
			}
		}
	}

	public void reproducir(string nombre)
	{
		foreach (Sonido sonido in sonidos)
		{
			if (sonido.Nombre == nombre)
			{
				sonido.Activo = true;
			}
		}
	}

	public void reproducir(string nombre, Vector2 emisor, Vector2 oyente)
	{
		foreach (Sonido sonido in sonidos)
		{
			if (sonido.Nombre == nombre)
			{
				float num = MathHelper.Distance(emisor.X, oyente.X);
				float num2 = MathHelper.Distance(emisor.Y, oyente.Y);
				float num3 = (num + num2) * 0.001f;
				if (1f - num3 <= 1f && 1f - num3 >= 0f)
				{
					sonido.instanciaSonido.Volume = 1f - num3;
				}
				sonido.Activo = true;
			}
		}
	}

	public void stop(string nombre)
	{
		foreach (Sonido sonido in sonidos)
		{
			if (sonido.Nombre == nombre)
			{
				sonido.Activo = false;
			}
		}
	}

	public void actualizar(GameTime tiempo)
	{
		foreach (Sonido sonido in sonidos)
		{
			sonido.play();
			sonido.Activo = false;
		}
		if (MostrarDatos)
		{
			elapsedTime += tiempo.ElapsedGameTime;
			if (elapsedTime > TimeSpan.FromSeconds(1.0))
			{
				elapsedTime -= TimeSpan.FromSeconds(1.0);
				frameRate = frameCounter;
				frameCounter = 0;
			}
		}
		if (Pause)
		{
			return;
		}
		foreach (Objeto objeto in objetos)
		{
			objeto.colisionSuperior = false;
			objeto.colisionInferior = false;
			objeto.colisionDerecha = false;
			objeto.colisionIzquierda = false;
		}
		foreach (Objeto objeto2 in objetos)
		{
			if (objeto2.Activo && objeto2.Tipo_ != Objeto.Tipo.Particulas)
			{
				Fisica.aplicarFisicas(objeto2, objetos, tiempo);
				objeto2.actualizar(tiempo);
				objeto2.actualizarSonidos();
			}
			if (objeto2.Tipo_ == Objeto.Tipo.Particulas)
			{
				Fisica.aplicarFisicas(objeto2, objetos, tiempo);
				objeto2.actualizar(tiempo);
				objeto2.actualizarSonidos();
			}
		}
	}

	public void dibujarObjetos(SpriteBatch spriteBatch, GraphicsDeviceManager graphics)
	{
		Pantalla.BeginDraw();
		graphics.GraphicsDevice.Clear(Color_);
		foreach (Objeto objeto in objetos)
		{
			if (objeto.Lugar_ == Objeto.Lugar.Fondo && objeto.Tipo_ == Objeto.Tipo.Particulas)
			{
				objeto.dibujar(spriteBatch, graphics);
			}
			else if (objeto.Lugar_ == Objeto.Lugar.Fondo && objeto.Activo)
			{
				objeto.dibujar(spriteBatch, graphics);
			}
		}
		foreach (Objeto objeto2 in objetos)
		{
			if (objeto2.Lugar_ == Objeto.Lugar.Medio && objeto2.Tipo_ == Objeto.Tipo.Particulas)
			{
				objeto2.dibujar(spriteBatch, graphics);
			}
			else if (objeto2.Lugar_ == Objeto.Lugar.Medio && objeto2.Activo)
			{
				objeto2.dibujar(spriteBatch, graphics);
			}
		}
		foreach (Objeto objeto3 in objetos)
		{
			if (objeto3.Lugar_ == Objeto.Lugar.Frente && objeto3.Tipo_ == Objeto.Tipo.Particulas)
			{
				objeto3.dibujar(spriteBatch, graphics);
			}
			else if (objeto3.Lugar_ == Objeto.Lugar.Frente && objeto3.Activo)
			{
				objeto3.dibujar(spriteBatch, graphics);
			}
		}
		spriteBatch.Begin();
		dibujarGrilla(spriteBatch);
		dibujarDatos(spriteBatch);
		spriteBatch.End();
	}

	private void dibujarGrilla(SpriteBatch spriteBatch)
	{
		if (!MostrarGrilla)
		{
			return;
		}
		for (int i = 0; i < 27; i++)
		{
			spriteBatch.Draw(pixel, new Rectangle(i * 30, 0, 1, 600), colorGrilla);
			for (int j = 0; j < 20; j++)
			{
				spriteBatch.Draw(pixel, new Rectangle(0, j * 30, 900, 1), colorGrilla);
			}
		}
	}

	private void dibujarDatos(SpriteBatch spriteBatch)
	{
		if (MostrarDatos)
		{
			frameCounter++;
			string text = "FPS: " + frameRate + "\nTamano: " + Pantalla.Tamaño + "\nInclinacion: " + Pantalla.Inclinacion + "\nPosicion: " + Pantalla.Posicion.ToString().Replace("{", "").Replace("}", "");
			spriteBatch.DrawString(fuente, text, new Vector2(32f, 32f), Color.White);
		}
	}

	public void crearMensaje(string nombre, string texto, string fuente, ContentManager content)
	{
		Objeto objeto = new Texto();
		objeto.Nombre = nombre;
		objeto.NombreTextura = fuente;
		objeto.Texto = texto;
		objeto.cargar(content);
	}

	public Objeto getObjetoPorNombre(string nombreObjeto)
	{
		foreach (Objeto objeto in objetos)
		{
			if (objeto.Nombre.Contains(nombreObjeto))
			{
				return objeto;
			}
		}
		return null;
	}

	public bool detectarColision(string objeto1, string objeto2)
	{
		if (getObjetoPorNombre(objeto1).Cuerpo.Intersects(getObjetoPorNombre(objeto2).Cuerpo))
		{
			return true;
		}
		return false;
	}

	public void respawn(string objeto, Vector2 posicion, GameTime gameTime, float tiempo)
	{
		if (getObjetoPorNombre(objeto).respawn < tiempo)
		{
			getObjetoPorNombre(objeto).respawn += (float)gameTime.ElapsedGameTime.TotalSeconds;
			return;
		}
		getObjetoPorNombre(objeto).respawn = 0f;
		getObjetoPorNombre(objeto).Posicion = posicion;
		getObjetoPorNombre(objeto).Cuerpo.X = (int)posicion.X;
		getObjetoPorNombre(objeto).Cuerpo.Y = (int)posicion.Y;
		getObjetoPorNombre(objeto).Activo = true;
	}

	public void hacerPause()
	{
		if (Pause)
		{
			Pause = false;
		}
		else
		{
			Pause = true;
		}
	}

	public void desactivarShaders()
	{
		ActivarShader = false;
		Shader_ = Objeto.Shader.Ninguno;
	}

	public void activarShaders(Objeto.Shader shader, ContentManager content)
	{
		ActivarShader = true;
		switch (shader)
		{
		case Objeto.Shader.Negative:
			PostProcessing = new Negative();
			PostProcessing.cargar(content);
			Shader_ = Objeto.Shader.Negative;
			break;
		case Objeto.Shader.Brightness:
			PostProcessing = new Brightness();
			PostProcessing.cargar(content);
			Shader_ = Objeto.Shader.Brightness;
			PostProcessing.intensidad = 1f;
			break;
		}
	}

	public bool IntersectPixels(string objeto1, string objeto2)
	{
		Rectangle bodyData = getObjetoPorNombre(objeto1).bodyData;
		Color[] textureData = getObjetoPorNombre(objeto1).TextureData;
		Rectangle bodyData2 = getObjetoPorNombre(objeto2).bodyData;
		Color[] textureData2 = getObjetoPorNombre(objeto2).TextureData;
		int num = Math.Max(bodyData.Top, bodyData2.Top);
		int num2 = Math.Min(bodyData.Bottom, bodyData2.Bottom);
		int num3 = Math.Max(bodyData.Left, bodyData2.Left);
		int num4 = Math.Min(bodyData.Right, bodyData2.Right);
		for (int i = num; i < num2; i++)
		{
			for (int j = num3; j < num4; j++)
			{
				Color color = textureData[j - bodyData.Left + (i - bodyData.Top) * bodyData.Width];
				Color color2 = textureData2[j - bodyData2.Left + (i - bodyData2.Top) * bodyData2.Width];
				if (color.A != 0 && color2.A != 0)
				{
					return true;
				}
			}
		}
		return false;
	}

	public static void guardarDatos(object[] datos)
	{
		//IL_0001: Unknown result type (might be due to invalid IL or missing references)
		//IL_0007: Expected O, but got Unknown
		XmlDocument val = new XmlDocument();
		XmlNode val2 = (XmlNode)(object)val.CreateElement("Datos");
		for (int i = 0; i < datos.Length; i++)
		{
			XmlAttribute val3 = val.CreateAttribute("dato" + i);
			((XmlNode)val3).InnerText = datos[i].ToString();
			val2.Attributes.Append(val3);
		}
		((XmlNode)val).AppendChild(val2);
		val.Save("Datos");
	}

	public static object[] cargarDatos()
	{
		//IL_0001: Unknown result type (might be due to invalid IL or missing references)
		//IL_0007: Expected O, but got Unknown
		XmlDocument val = new XmlDocument();
		val.Load("Datos");
		object[] array = new object[((XmlNamedNodeMap)((XmlNode)val).FirstChild.Attributes).Count];
		for (int i = 0; i < ((XmlNamedNodeMap)((XmlNode)val).FirstChild.Attributes).Count; i++)
		{
			array[i] = ((XmlNode)((XmlNode)val).FirstChild.Attributes[i]).InnerText;
		}
		return array;
	}
}
