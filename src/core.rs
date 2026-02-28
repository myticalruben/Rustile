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
    MoveFocus(i32),       // Cambiar de ventana
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

    pub fn rotate_focus(&mut self, direction: i32) {
        if self.clients.is_empty() {
            return;
        }

        // 1. Encontrar posicion actual
        let current_pos = self
            .clients
            .iter()
            .position(|&id| id == self.focused)
            .unwrap_or(0);

        let len = self.clients.len() as i32;
        let next_pos = (current_pos as i32 + direction).rem_euclid(len) as usize;

        //3. Actualizar el ID enfocado
        self.focused = self.clients[next_pos];
    }
}

pub mod mods {
    pub const MOD_4: u16 = 64; // Tecla Super/Windows
    pub const SHIFT: u16 = 1; // Shift
    pub const CONTROL: u16 = 4; // Ctrl
    pub const ALT: u16 = 8; // Alt
}
