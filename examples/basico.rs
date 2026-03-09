use rustile::{RustileConfig, server::RustileServer};

fn main(){
    let config = RustileConfig::default();
    let wm = RustileServer::new(config);

    if let Err(e) = wm.run(){
        eprintln!("Error fatal: {}", e)
    }
}