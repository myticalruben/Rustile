use xkeysym::Keysym;

pub type WindowId = u32;

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug)]
pub struct Stack {
    pub focused: WindowId,
    pub clients: Vec<WindowId>,
}

#[derive(Debug)]
pub struct Workspace {
    pub name: String,
    pub stack: Stack,
}

#[derive(Debug, Clone)]
pub enum Action {
    Spawn(String),        //Lanzar un comando (ej. alacritty)
    KillClient,           // Cerrar ventana actual
    MoveFocus(i16),       // Cambiar de ventana
    GoToWorkspace(usize), // Cambiar de workspace
}

pub struct KeyBinding {
    pub modifiers: u16, // Alt, Super, Control...
    pub key: Keysym,    // "Return", "q", "space"...
    pub action: Action,
}

impl Stack {
    pub fn add(&mut self, win: WindowId) {
        self.clients.push(win);
        self.focused = win;
    }
}

pub mod mods {
    pub const MOD_4: u16 = 64; // Tecla Super/Windows
    pub const SHIFT: u16 = 1; // Shift
    pub const CONTROL: u16 = 4; // Ctrl
    pub const ALT: u16 = 8; // Alt
}
