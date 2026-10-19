// Sensores del dispositivo en la versión web (las funciones están en web/index.html).

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn museo_beta() -> f64;
}

/// La inclinación de delante hacia atrás del dispositivo en grados (DeviceOrientationEvent.beta,
/// lo que Construct 2 llama Touch.Beta). 0 en PC o si el navegador no la da.
pub fn beta() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { museo_beta() }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}
