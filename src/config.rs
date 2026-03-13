use std::collections::HashMap;

use bitflags::bitflags;
use xkeysym::Keysym;

#[derive(Debug, Clone, )]
pub enum Action{
    Spawn(String),
    Quit
}

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Modifiers: u32{
        const NONE  = 0b0000;
        const SUPER = 0b0001;
        const ALT   = 0b0010;
        const CTRL  = 0b0100;
        const SHIFT = 0b1000;
    }  
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut{
    pub modifier: Modifiers,
    pub key: Keysym,
}

#[derive(Debug, Clone)]
pub struct RustileConfig {
    pub border_width: u32,
    pub color_focus: u32,
    pub color_normal: u32,
    pub gap_size: u32,
    pub workspaces: Vec<String>,
    pub shortcuts: HashMap<Shortcut, Action>,
}

impl RustileConfig {
    pub fn set_shortcut(&mut self, keys: HashMap<Shortcut, Action>){
        self.shortcuts = keys;
    }
}

impl Default for RustileConfig {
    fn default() -> Self {

        let mut shortcuts = HashMap::new();
        shortcuts.insert(Shortcut {modifier: Modifiers::SUPER, key: Keysym::Return}, Action::Spawn("weston-terminal".to_string()));
        shortcuts.insert(Shortcut { modifier: Modifiers::SUPER, key: Keysym::q }, Action::Quit);

        Self {
            border_width: 2,
            color_focus: 0xffaa00,
            color_normal: 0x444444,
            gap_size: 5,
            workspaces: vec!["1".into(),"2".into(),"3".into(),"4".into()],
            shortcuts,
        }
    }
}
