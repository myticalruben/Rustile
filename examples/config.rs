use rustile::{Rustile, core::Action, core::KeyBinding, mods};
use x11rb::connect;
use xkeysym::{Keysym, key};

fn main() {
    // 1. Conexion al servidor X 11
    let (conn, screen_num) = connect(None).expect("No se pudo conectar a X11");

    // 2. Iniciar el WM
    let mut wm = Rustile::new(conn, screen_num);

    // 3. Definir combinaciones de prueba
    // Usamos las constantes nativos de xkeysym
    let test = vec![
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::Return,
            action: Action::Spawn("xterm".into()),
        },
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::Q,
            action: Action::KillClient,
        },
    ];

    wm.setup_keybindings(test);

    if let Err(e) = wm.run() {
        eprintln!("Error en el WM: {}", e);
    }
}
