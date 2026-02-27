pub type WindowId = u32;

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct Stack {
    pub focused: WindowId,
    pub clients: Vec<WindowId>,
}

pub struct Workspace {
    pub name: String,
    pub stack: Option<Stack>,
}

impl Stack {
    pub fn add(&mut self, win: WindowId) {
        self.clients.push(win);
        self.focused = win;
    }
}
