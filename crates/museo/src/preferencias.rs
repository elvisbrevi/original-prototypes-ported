// Preferencias que sobreviven al juego (PlayerPrefs de Unity, por ejemplo un récord). En la web
// van al localStorage del navegador (funciones del index.html); en PC, a un archivo de texto en
// la carpeta de datos del usuario (`$XDG_DATA_HOME/museo/<juego>.txt`, `~/.local/share/...`,
// `%APPDATA%\museo\...` o `~/Library/Application Support/museo/...`). Solo guardan enteros.

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn museo_pref_leer(clave: i32, defecto: i32) -> i32;
    fn museo_pref_guardar(clave: i32, valor: i32);
}

pub struct Preferencias {
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    juego: String,
}

/// FNV-1a de 32 bits: la clave de localStorage es un número (la página le agrega su título).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn hash(texto: &str) -> i32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in texto.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h as i32
}

#[cfg(not(target_arch = "wasm32"))]
impl Preferencias {
    fn archivo(&self) -> Option<std::path::PathBuf> {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::var_os("APPDATA").map(std::path::PathBuf::from))
            .or_else(|| {
                let home = std::path::PathBuf::from(std::env::var_os("HOME")?);
                Some(if cfg!(target_os = "macos") {
                    home.join("Library/Application Support")
                } else {
                    home.join(".local/share")
                })
            })?;
        Some(base.join("museo").join(format!("{}.txt", self.juego)))
    }

    fn todas(&self) -> Vec<(String, i32)> {
        let Some(texto) = self.archivo().and_then(|a| std::fs::read_to_string(a).ok()) else {
            return Vec::new();
        };
        texto
            .lines()
            .filter_map(|l| {
                let (k, v) = l.split_once('=')?;
                Some((k.to_owned(), v.trim().parse().ok()?))
            })
            .collect()
    }
}

impl Preferencias {
    /// Las preferencias de un juego (su slug).
    pub fn de(juego: &str) -> Self {
        Self {
            juego: juego.to_owned(),
        }
    }

    /// PlayerPrefs.GetInt: el valor guardado, o `defecto`.
    pub fn leer_entero(&self, clave: &str, defecto: i32) -> i32 {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { museo_pref_leer(hash(clave), defecto) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.todas()
                .into_iter()
                .find(|(k, _)| k == clave)
                .map_or(defecto, |(_, v)| v)
        }
    }

    /// PlayerPrefs.SetInt (se escribe enseguida, como el Web Player).
    pub fn guardar_entero(&self, clave: &str, valor: i32) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            museo_pref_guardar(hash(clave), valor)
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut todas = self.todas();
            todas.retain(|(k, _)| k != clave);
            todas.push((clave.to_owned(), valor));
            if let Some(archivo) = self.archivo() {
                let texto: String = todas.iter().map(|(k, v)| format!("{k}={v}\n")).collect();
                if let Some(carpeta) = archivo.parent() {
                    let _ = std::fs::create_dir_all(carpeta);
                }
                let _ = std::fs::write(archivo, texto);
            }
        }
    }
}
