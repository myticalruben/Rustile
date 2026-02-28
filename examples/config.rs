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
            action: Action::Spawn("alacritty".into()),
        },
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::b,
            action: Action::Spawn("brave-browser".into()),
        },
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::q,
            action: Action::KillClient,
        },
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::h,
            action: Action::MoveFocus(-1),
        },
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::l,
            action: Action::MoveFocus(1),
        },
    ];

    wm.setup_keybindings(test);

    if let Err(e) = wm.run() {
        eprintln!("Error en el WM: {}", e);
    }
}
