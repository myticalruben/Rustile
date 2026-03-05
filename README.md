# 🦀 Rustile

**Rustile** es un gestor de ventanas dinámico (Tiling Window Manager) inspirado en Qtile, escrito completamente en **Rust** utilizando el protocolo X11 mediante la biblioteca `x11rb`. 



## ✨ Características

* **Tiling Dinámico:** Organización automática de ventanas en columnas y stacks.
* **Seguridad de Memoria:** Gracias a Rust, olvida los errores de segmentación comunes en otros WMs.
* **Configuración en Rust:** Configura tu WM con la potencia de un lenguaje compilado.
* **Protocolo Moderno:** Implementación de `WM_DELETE_WINDOW` para cierres elegantes de aplicaciones.
* **Ventanas Flotantes:** Detección automática de diálogos y utilidades mediante `_NET_WM_WINDOW_TYPE`.
* **Ligero y Rápido:** Sin dependencias pesadas, usando `xkeysym` para una gestión de teclado eficiente y tipada.

## 🚀 Inicio Rápido

### Requisitos previos
Necesitas tener instalado Rust y las bibliotecas de desarrollo de X11 en tu sistema:

```bash
# En Debian/Ubuntu
sudo apt install libx11-dev
```
```toml
[dependencies]
rustile = "0.0.4"
x11rb = { version = "0.13", traits = ["all"] }
xkeysym = "0.2"
```

## 🛠 Ejemplo de Configuración
Crea un archivo examples/config.rs o usa tu main.rs para definir la lógica de tu escritorio:

```Rust
use x11rb::connect;
use xkeysym::key;
use rustile::{Rustile, core::{KeyBinding, Action, mods}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Conectar al servidor X
    let (conn, screen_num) = connect(None)?;
    
    // 2. Inicializar Rustile
    let mut wm = Rustile::new(conn, screen_num);

    // 3. Configurar Atajos de Teclado (Usando constantes nativas)
    let bindings = vec![
        KeyBinding {
            modifiers: mods::MOD_4, // Tecla Super/Windows
            key: key::RETURN,
            action: Action::Spawn("xterm".into()),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: key::J,
            action: Action::MoveFocus(1),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: key::Q,
            action: Action::KillClient,
        },
    ];

    // 4. Cargar configuración y ejecutar
    wm.setup_keybindings(bindings);
    wm.run()?;
    
    Ok(())
}
```
## ⌨️ Atajos Sugeridos

| Combinación     |    Acción  |
|-----------------|------------|
|Win + Enter,Abrir|    Terminal|
|Win + J,         |    siguiente ventana|
|Win + K,         |    ventana anterior|
|Win + Shift + J, |    Intercambiar ventana con la siguiente|
|Win + Shift + Q, |    Cerrar ventana actual (Elegante)|
|Win + [1-9],     |    Cambiar de Workspace|


## 🗺️ Hoja de Ruta (Roadmap)
[x] Gestión de teclado con xkeysym.

[x] Soporte para el protocolo WM_DELETE_WINDOW.

[x] Detección de ventanas flotantes.

[ ] Soporte para múltiples monitores (Xinerama/RandR).

[x] Intercambio físico de ventanas (Swap).

[ ] Barra de estado (StatusBar) integrada.

## 🤝 Contribuciones
¡Las contribuciones son bienvenidas! Siéntete libre de abrir un Issue o enviar un Pull Request.
Este proyecto busca ser una base sólida y educativa para quienes deseen entender cómo funcionan los Window Managers desde cero con Rust.
Licencia: MIT o Apache-2.0.