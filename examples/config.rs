mod keymaps;

use rustile::{core::RustileConfig, rustile::Rustile};
use x11rb::connect;

use crate::keymaps::keymaps;

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

    wm.set_background_color(0xfff237).unwrap();
    wm.set_config(config);

    // 3. Definir combinaciones de prueba
    // Usamos las constantes nativos de xkeysym
    wm.setup_keybindings(keymaps());

    //4. ejecutamos el wm
    if let Err(e) = wm.run() {
        eprintln!("Error en el WM: {}", e);
    }
}
