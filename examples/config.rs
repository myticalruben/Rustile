use rustile::{core::Action, core::KeyBinding, mods, rustile::Rustile};
use x11rb::connect;
use xkeysym::{Keysym, key};

fn main() {
    // 1. Conexion al servidor X 11
    let (conn, screen_num) = connect(None).expect("No se pudo conectar a X11");

    // 2. Iniciar el WM
    let mut wm = Rustile::new(conn, screen_num);
    let color: u32 = 0xffff25;
    let _ = wm.set_background_color(color);

    // 3. Definir combinaciones de prueba
    // Usamos las constantes nativos de xkeysym
    let mut test = vec![
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
        KeyBinding {
            modifiers: mods::ALT | mods::SHIFT,
            key: Keysym::h,
            action: Action::Swap(-1),
        },
        KeyBinding {
            modifiers: mods::ALT | mods::SHIFT,
            key: Keysym::l,
            action: Action::Swap(1),
        },
        KeyBinding {
            modifiers: mods::ALT | mods::CONTROL,
            key: Keysym::h,
            action: Action::ChangeRatio(0.05),
        },
        KeyBinding {
            modifiers: mods::ALT | mods::CONTROL,
            key: Keysym::l,
            action: Action::ChangeRatio(-0.05),
        },
    ];

    for i in 0..9 {
        let key_val = u32::from(Keysym::_1) + i;
        let k = xkeysym::Keysym::from(key_val);

        test.push(KeyBinding {
            modifiers: mods::ALT,
            key: k,
            action: Action::GoToWorkspace(i as usize),
        });
    }

    wm.setup_keybindings(test);

    if let Err(e) = wm.run() {
        eprintln!("Error en el WM: {}", e);
    }
}
