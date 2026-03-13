use std::collections::HashMap;

use rustile::{Action, RustileConfig, Shortcut, server::RustileServer};

fn main(){
    let mut wm = RustileServer::new(RustileConfig::default());

    let mut keys = HashMap::new();
    keys.insert( Shortcut {modifier: 64, key: 36 }, Action::Spawn("gedit".to_string()));
    keys.insert(Shortcut {modifier: 64, key: 24}, Action::Quit);

    let mut config = RustileConfig::default();
    config.set_shortcut(keys);

    wm.set_config(config);

    if let Err(e) = wm.run(){
        eprintln!("Error fatal: {}", e)
    }
}