use rustile::{
    core::{Action, KeyBinding, RustileConfig},
    mods,
    rustile::Rustile,
};
use x11rb::connect;
use xkeysym::Keysym;

fn main() {
    // 1. Conexion al servidor X 11
    let (conn, screen_num) = connect(None).expect("No se pudo conectar a X11");

    // 2. Estableceemos las configuraciones iniciales
    let mut wm = Rustile::new(conn, screen_num);

    let config = RustileConfig {
        border_width: 2,
        color_normal: 0xbffffff,
        color_focus: 0xbf33f22,
        gap_size: 4,
    };

    wm.set_background_color(0xffff25).unwrap();
    wm.set_config(config);

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
            action: Action::Swap(1),
        },
        KeyBinding {
            modifiers: mods::ALT | mods::SHIFT,
            key: Keysym::l,
            action: Action::Swap(-1),
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
        KeyBinding {
            modifiers: mods::ALT,
            key: Keysym::r,
            action: Action::Restart,
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
        test.push(KeyBinding {
            modifiers: mods::ALT | mods::SHIFT,
            key: k,
            action: Action::MoveToWorkspace(i as usize),
        });
    }

    wm.setup_keybindings(test);

    //4. ejecutamos el wm
    if let Err(e) = wm.run() {
        eprintln!("Error en el WM: {}", e);
    }
}
