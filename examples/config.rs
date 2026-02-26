use rustile::Rustile;
use x11rb::connect;

fn main() {
    let (conn, screen_num) = connect(None).expect("No se pudo conectar a X11");

    let mut my_wm = Rustile::new(conn, screen_num);

    if let Err(e) = my_wm.run() {
        eprintln!("Error en el WM: {}", e);
    }
}
