use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Action{
    Spawn(String),
    Quit
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut{
    pub modifier: u32,
    pub key: u32,
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
        shortcuts.insert(Shortcut {modifier: 64, key: 36}, Action::Spawn("weston-terminal".to_string()));
        shortcuts.insert(Shortcut { modifier: 64, key: 24 }, Action::Quit);

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
