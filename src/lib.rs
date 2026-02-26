use x11rb::connection::Connection;

use crate::core::Workspace;

mod core;

pub struct Rustile<C: Connection> {
    conn: C,
    screen_num: usize,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

impl<C: Connection> Rustile<C> {
    pub fn new(conn: C, screen_num: usize) -> Self {
        Self {
            conn,
            screen_num,
            workspaces: vec![Workspace {
                name: "1".to_string(),
                stack: None,
            }],
            current_workspace: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Rustile iniciando...");
        Ok(())
    }
}
