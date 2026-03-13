use std::collections::HashMap;

use rustile::{Action, RustileConfig, Shortcut, server::RustileServer, Modifiers};
use xkeysym::Keysym;

fn main(){
    let mut wm = RustileServer::new(RustileConfig::default());

    let mut keys = HashMap::new();
    keys.insert( Shortcut {modifier: Modifiers::ALT | Modifiers::SHIFT, key: Keysym::Return }, Action::Spawn("weston-terminal".to_string()));
    keys.insert(Shortcut {modifier: Modifiers::ALT, key: Keysym::q}, Action::Quit);

    let mut config = RustileConfig::default();
    config.set_shortcut(keys);

    wm.set_config(config);

    if let Err(e) = wm.run(){
        eprintln!("Error fatal: {}", e)
    }
}