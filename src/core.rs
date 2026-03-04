use std::usize;

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
pub struct Layout {
    pub ratio: f32,
}

#[derive(Debug)]
pub struct Stack {
    pub focused: WindowId,
    pub clients: Vec<WindowId>,
}

#[derive(Debug)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
    pub stack: Stack,
    pub layout: Layout,
}

#[derive(Debug, Clone)]
pub enum Action {
    Spawn(String), //Lanzar un comando (ej. alacritty)
    Swap(i32),
    KillClient, // Cerrar ventana actual
    ChangeRatio(f32),
    MoveFocus(i32),         // Cambiar de ventana
    GoToWorkspace(usize),   // Cambiar de workspace
    MoveToWorkspace(usize), // Cambiar de workspace
}

pub struct KeyBinding {
    pub modifiers: u16, // Alt, Super, Control...
    pub key: Keysym,    // "Return", "q", "space"...
    pub action: Action,
}

impl Workspace {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            stack: Stack::new(),
            layout: Layout::new(),
        }
    }
}

impl Layout {
    pub fn new() -> Self {
        Self { ratio: 0.5 } //Empezamos al 50/50
    }

    pub fn change_ratio(&mut self, delta: f32) {
        //Limitamos el ratio para que ninguna columna desaparezca
        self.ratio = (self.ratio + delta).clamp(0.1, 0.9);
    }
}

impl Stack {
    pub fn new() -> Self {
        Self {
            focused: 0,
            clients: Vec::new(),
        }
    }

    pub fn add(&mut self, win: WindowId) {
        self.clients.push(win);
        self.focused = win;
    }

    pub fn swap(&mut self, direction: i32) {
        let len = self.clients.len();
        if len < 2 {
            return;
        }

        //1. Buscamos el indice de la ventana que tiene el foco
        if let Some(pos) = self.clients.iter().position(|&id| id == self.focused) {
            //2. Calculamos el indice de destino con aritmetica modular
            let target = (pos as i32 + direction).rem_euclid(len as i32) as usize;

            //3. Intercambiamos las posiciones en el vector
            self.clients.swap(pos, target);
        }
    }

    pub fn swap_focus(&mut self, direction: i32) {
        if self.clients.len() < 2 {
            return;
        }

        //1. Encontrar la posicion de la ventana enfocada
        let current_pos = self
            .clients
            .iter()
            .position(|&id| id == self.focused)
            .unwrap_or(0);

        //2. Calcular la posicion de destino
        let len = self.clients.len() as i32;
        let target_pos = (current_pos as i32 + direction).rem_euclid(len) as usize;

        //3. Intercambiar fisicamente en el vector
        self.clients.swap(current_pos, target_pos);
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
