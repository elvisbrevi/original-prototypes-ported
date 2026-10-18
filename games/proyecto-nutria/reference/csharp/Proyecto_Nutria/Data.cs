using System.Xml;

namespace Proyecto_Nutria;

internal class Data
{
	private static XmlDocument doc;

	public Data()
	{
		//IL_0008: Unknown result type (might be due to invalid IL or missing references)
		//IL_0012: Expected O, but got Unknown
		doc = new XmlDocument();
		doc.Load("..\\..\\..\\Datos\\Save.xml");
	}

	public static int cargar()
	{
		doc.Load("..\\..\\..\\Datos\\Save.xml");
		return int.Parse(((XmlNode)doc).FirstChild.ChildNodes[0].ChildNodes[0].InnerText);
	}

	public static void guardar(int idNivel)
	{
		doc.Load("..\\..\\..\\Datos\\Save.xml");
		((XmlNode)doc).FirstChild.ChildNodes[0].ChildNodes[0].InnerText = idNivel.ToString();
		doc.Save("..\\..\\..\\Datos\\Save.xml");
	}
}
