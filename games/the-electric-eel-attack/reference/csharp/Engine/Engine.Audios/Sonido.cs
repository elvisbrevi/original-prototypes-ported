using Microsoft.Xna.Framework.Audio;
using Microsoft.Xna.Framework.Content;

namespace Engine.Audios;

public class Sonido
{
	public SoundEffect sonido { get; set; }

	public SoundEffectInstance instanciaSonido { get; set; }

	public bool Repetir { get; set; }

	public bool Activo { get; set; }

	public string Nombre { get; set; }

	public float Volumen { get; set; }

	public void cargar(ContentManager content, string nombre)
	{
		Nombre = nombre;
		sonido = content.Load<SoundEffect>("Audios/" + nombre);
		instanciaSonido = sonido.CreateInstance();
		instanciaSonido.IsLooped = false;
		instanciaSonido.Volume = Volumen;
	}

	public void play()
	{
		if (Activo)
		{
			instanciaSonido.Play();
		}
	}

	public void pause()
	{
		if (instanciaSonido.State == SoundState.Playing)
		{
			instanciaSonido.Pause();
		}
	}

	public void resume()
	{
		if (instanciaSonido.State == SoundState.Paused)
		{
			instanciaSonido.Resume();
		}
	}

	public void stop()
	{
		if (instanciaSonido.State == SoundState.Paused || instanciaSonido.State == SoundState.Playing)
		{
			instanciaSonido.Stop();
			Activo = false;
		}
	}
}
